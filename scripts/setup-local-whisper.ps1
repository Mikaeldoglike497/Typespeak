param(
  [switch]$Force,
  [switch]$CpuOnly
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$runtimeDir = Join-Path $projectRoot "runtime"
$cudaRuntimeDir = Join-Path $projectRoot "runtime-cuda"
$modelsDir = Join-Path $projectRoot "models"
$whisperExe = Join-Path $runtimeDir "whisper-cli.exe"
$cudaWhisperExe = Join-Path $cudaRuntimeDir "whisper-cli.exe"
$modelName = "ggml-large-v3-turbo-q5_0.bin"
$modelPath = Join-Path $modelsDir $modelName
$runtimeUrl = "https://github.com/ggml-org/whisper.cpp/releases/download/v1.9.1/whisper-bin-x64.zip"
$cudaRuntimeUrl = "https://github.com/ggml-org/whisper.cpp/releases/download/v1.9.1/whisper-cublas-12.4.0-bin-x64.zip"
$cudaRuntimeSha256 = "106a2030eff8998e4ef320fe72e263a78449e9040386ee27c41ea80b001b601b"
$modelUrl = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/$modelName"
$modelSha256 = "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2"
$tempDir = Join-Path ([IO.Path]::GetTempPath()) ("typespeak-setup-" + [Guid]::NewGuid())

function Download-File {
  param(
    [Parameter(Mandatory = $true)][string]$Uri,
    [Parameter(Mandatory = $true)][string]$Destination
  )

  & curl.exe --fail --location --retry 3 --output $Destination $Uri
  if ($LASTEXITCODE -eq 35) {
    Write-Host "Windows could not reach its certificate revocation service; retrying with standard certificate validation only."
    & curl.exe --ssl-no-revoke --fail --location --retry 3 --output $Destination $Uri
  }
  if ($LASTEXITCODE -ne 0) {
    throw "Download failed with curl exit code $LASTEXITCODE."
  }
}

function Test-NvidiaGpu {
  if (-not (Get-Command "nvidia-smi.exe" -ErrorAction SilentlyContinue)) {
    return $false
  }
  & nvidia-smi.exe -L *> $null
  return $LASTEXITCODE -eq 0
}

$installCuda = -not $CpuOnly -and (Test-NvidiaGpu)
$requiredDirectories = @($runtimeDir, $modelsDir, $tempDir)
if ($installCuda) {
  $requiredDirectories += $cudaRuntimeDir
}
New-Item -ItemType Directory -Force -Path $requiredDirectories | Out-Null

try {
  if ($Force -or -not (Test-Path -LiteralPath $whisperExe)) {
    $runtimeArchive = Join-Path $tempDir "whisper-bin-x64.zip"
    Write-Host "Downloading whisper.cpp v1.9.1 runtime..."
    Download-File -Uri $runtimeUrl -Destination $runtimeArchive
    $runtimeExtract = Join-Path $tempDir "runtime"
    Expand-Archive -LiteralPath $runtimeArchive -DestinationPath $runtimeExtract -Force
    $downloadedExe = Get-ChildItem -LiteralPath $runtimeExtract -Recurse -File -Filter "whisper-cli.exe" |
      Select-Object -First 1
    if (-not $downloadedExe) {
      throw "whisper-cli.exe was not found in the official runtime archive."
    }
    Get-ChildItem -LiteralPath $downloadedExe.Directory.FullName -File |
      Copy-Item -Destination $runtimeDir -Force
  }

  if ($installCuda -and ($Force -or -not (Test-Path -LiteralPath $cudaWhisperExe))) {
    $cudaArchive = Join-Path $tempDir "whisper-cublas-12.4.0-bin-x64.zip"
    Write-Host "NVIDIA GPU found. Downloading whisper.cpp CUDA runtime (~678 MB)..."
    Download-File -Uri $cudaRuntimeUrl -Destination $cudaArchive
    $actualCudaSha256 = (Get-FileHash -LiteralPath $cudaArchive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualCudaSha256 -ne $cudaRuntimeSha256) {
      throw "CUDA runtime checksum mismatch. The downloaded files were not installed."
    }
    $cudaExtract = Join-Path $tempDir "runtime-cuda"
    Expand-Archive -LiteralPath $cudaArchive -DestinationPath $cudaExtract -Force
    $downloadedCudaExe = Get-ChildItem -LiteralPath $cudaExtract -Recurse -File -Filter "whisper-cli.exe" |
      Select-Object -First 1
    if (-not $downloadedCudaExe) {
      throw "whisper-cli.exe was not found in the official CUDA runtime archive."
    }
    Get-ChildItem -LiteralPath $downloadedCudaExe.Directory.FullName -File |
      Copy-Item -Destination $cudaRuntimeDir -Force
  }

  $modelIsValid = (Test-Path -LiteralPath $modelPath) -and
    ((Get-Item -LiteralPath $modelPath).Length -gt 500MB) -and
    ((Get-FileHash -LiteralPath $modelPath -Algorithm SHA256).Hash.ToLowerInvariant() -eq $modelSha256)
  if ($Force -or -not $modelIsValid) {
    $partialModel = Join-Path $tempDir "$modelName.download"
    Write-Host "Downloading Whisper large-v3-turbo Q5_0 (~574 MB)..."
    Download-File -Uri $modelUrl -Destination $partialModel
    $actualSha256 = (Get-FileHash -LiteralPath $partialModel -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualSha256 -ne $modelSha256) {
      throw "Model checksum mismatch. The downloaded file was not installed."
    }
    Move-Item -LiteralPath $partialModel -Destination $modelPath -Force
  }

  Write-Host ""
  Write-Host "TypeSpeak local speech engine is ready."
  Write-Host "Runtime: $whisperExe"
  if ($installCuda) {
    Write-Host "CUDA:    $cudaWhisperExe"
  }
  Write-Host "Model:   $modelPath"
} finally {
  $resolvedTemp = [IO.Path]::GetFullPath($tempDir)
  $resolvedTempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
  $safeTemporaryPath = $resolvedTemp.StartsWith(
    $resolvedTempRoot,
    [StringComparison]::OrdinalIgnoreCase
  ) -and ([IO.Path]::GetFileName($resolvedTemp)).StartsWith("typespeak-setup-")
  if ($safeTemporaryPath -and (Test-Path -LiteralPath $resolvedTemp)) {
    Remove-Item -LiteralPath $resolvedTemp -Recurse -Force
  }
}
