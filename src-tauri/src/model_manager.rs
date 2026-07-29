use reqwest::blocking::{Client, Response};
use reqwest::header::RANGE;
use reqwest::StatusCode;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const DOWNLOAD_EVENT: &str = "typespeak://model-download";
const DOWNLOAD_CHUNK_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy)]
pub struct CatalogModel {
    pub id: &'static str,
    pub file_name: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub bytes: u64,
}

const CATALOG: [CatalogModel; 4] = [
    CatalogModel {
        id: "cohere-local",
        file_name: "cohere-transcribe-arabic-q4_k-imatrix.gguf",
        url: "https://huggingface.co/cstr/cohere-transcribe-arabic-07-2026-GGUF/resolve/main/cohere-transcribe-arabic-q4_k-imatrix.gguf",
        sha256: "18515fbd86d76f27026ad6414eaf3989edc868b8ca54c1cb05e62f52d67a2edc",
        bytes: 1_510_365_312,
    },
    CatalogModel {
        id: "qwen-local",
        file_name: "qwen3-asr-0.6b-q4_k.gguf",
        url: "https://huggingface.co/cstr/qwen3-asr-0.6b-GGUF/resolve/main/qwen3-asr-0.6b-q4_k.gguf",
        sha256: "f63771c02dfa486d9399d41ab6ab8cd2d8ca24e077cd32130ea1f67f4fd8dade",
        bytes: 631_026_336,
    },
    CatalogModel {
        id: "omni-local",
        file_name: "omniasr-ctc-300m-v2-q4_k.gguf",
        url: "https://huggingface.co/cstr/omniASR-CTC-300M-v2-GGUF/resolve/main/omniasr-ctc-300m-v2-q4_k.gguf",
        sha256: "cac0ae5eef46f146e47a1445e9fb8a4e894fac78c5b90e4d6318dc3734b34808",
        bytes: 203_542_816,
    },
    CatalogModel {
        id: "translator-m2m100",
        file_name: "m2m100-418m-q8_0.gguf",
        url: "https://huggingface.co/cstr/m2m100-418m-GGUF/resolve/main/m2m100-418m-q8_0.gguf",
        sha256: "2831c0ef471776acacbac2bc763efbd2e4853309f49f317635f288e161c8d921",
        bytes: 526_331_008,
    },
];

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomModelInstallRequest {
    pub model_id: String,
    pub download_url: String,
    pub file_name: String,
    pub expected_sha256: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedModelStatus {
    pub id: String,
    pub file_name: String,
    pub installed: bool,
    pub bytes: u64,
    pub expected_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgress {
    id: String,
    stage: &'static str,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    percent: Option<u8>,
}

#[derive(Clone, Copy)]
struct DownloadSpec<'a> {
    id: &'a str,
    url: &'a str,
    destination: &'a Path,
    expected_bytes: Option<u64>,
    expected_sha256: Option<&'a str>,
}

struct PreparedDownload {
    response: Response,
    starting_bytes: u64,
    total_bytes: Option<u64>,
    resuming: bool,
}

#[derive(Clone, Copy)]
struct PartialFile<'a> {
    path: &'a Path,
    bytes: u64,
}

struct TransferState {
    writer: BufWriter<File>,
    hasher: Sha256,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    last_event: Instant,
}

pub fn catalog_entry(model_id: &str) -> Option<CatalogModel> {
    CATALOG.iter().copied().find(|model| model.id == model_id)
}

pub fn catalog_status(model_id: &str) -> Result<ManagedModelStatus, String> {
    let model =
        catalog_entry(model_id).ok_or_else(|| format!("Unknown managed model: {model_id}"))?;
    Ok(file_status(
        model.id,
        model.file_name,
        Some(model.bytes),
        &catalog_model_path(model.file_name),
    ))
}

pub fn custom_status(file_name: &str) -> Result<ManagedModelStatus, String> {
    let safe_name = validate_model_file_name(file_name)?;
    Ok(file_status(
        file_name,
        &safe_name,
        None,
        &custom_model_path(&safe_name),
    ))
}

pub fn install_catalog(app: &AppHandle, model_id: &str) -> Result<ManagedModelStatus, String> {
    let model =
        catalog_entry(model_id).ok_or_else(|| format!("Unknown managed model: {model_id}"))?;
    let destination = catalog_model_path(model.file_name);
    if is_complete_file(&destination, Some(model.bytes)) {
        return catalog_status(model_id);
    }
    download_model(
        app,
        DownloadSpec {
            id: model.id,
            url: model.url,
            destination: &destination,
            expected_bytes: Some(model.bytes),
            expected_sha256: Some(model.sha256),
        },
    )?;
    catalog_status(model_id)
}

pub fn install_custom(
    app: &AppHandle,
    request: &CustomModelInstallRequest,
) -> Result<ManagedModelStatus, String> {
    validate_model_id(&request.model_id)?;
    let safe_name = validate_model_file_name(&request.file_name)?;
    normalize_download_url(&request.download_url)?;
    let checksum = request
        .expected_sha256
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(validate_sha256)
        .transpose()?;
    let destination = custom_model_path(&safe_name);
    download_model(
        app,
        DownloadSpec {
            id: &request.model_id,
            url: &request.download_url,
            destination: &destination,
            expected_bytes: None,
            expected_sha256: checksum.as_deref(),
        },
    )?;
    custom_status(&safe_name)
}

pub fn catalog_model_path_by_id(model_id: &str) -> Option<PathBuf> {
    catalog_entry(model_id).map(|model| catalog_model_path(model.file_name))
}

pub fn custom_model_path_by_name(file_name: &str) -> Result<PathBuf, String> {
    let safe_name = validate_model_file_name(file_name)?;
    Ok(custom_model_path(&safe_name))
}

pub fn managed_models_root() -> PathBuf {
    env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("TypeSpeak")
        .join("models")
}

fn catalog_model_path(file_name: &str) -> PathBuf {
    managed_models_root().join(file_name)
}

fn custom_model_path(file_name: &str) -> PathBuf {
    managed_models_root().join("custom").join(file_name)
}

fn file_status(
    id: &str,
    file_name: &str,
    expected_bytes: Option<u64>,
    path: &Path,
) -> ManagedModelStatus {
    let bytes = path.metadata().map(|metadata| metadata.len()).unwrap_or(0);
    ManagedModelStatus {
        id: id.to_owned(),
        file_name: file_name.to_owned(),
        installed: is_complete_file(path, expected_bytes),
        bytes,
        expected_bytes,
    }
}

fn is_complete_file(path: &Path, expected_bytes: Option<u64>) -> bool {
    path.metadata().is_ok_and(|metadata| {
        metadata.is_file()
            && expected_bytes
                .map(|expected| metadata.len() == expected)
                .unwrap_or(metadata.len() > 1_024)
    })
}

fn download_model(app: &AppHandle, spec: DownloadSpec<'_>) -> Result<(), String> {
    normalize_download_url(spec.url)?;
    let parent = spec
        .destination
        .parent()
        .ok_or_else(|| "The managed model destination is invalid.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create the model directory: {error}"))?;
    let partial = spec.destination.with_extension("gguf.part");
    download_to_partial(app, spec, &partial).map_err(|error| retryable_error(error, &partial))?;
    finish_install(&partial, spec.destination)?;
    emit_progress(
        app,
        spec.id,
        "installed",
        spec.expected_bytes.unwrap_or(0),
        spec.expected_bytes,
    );
    Ok(())
}

fn retryable_error(error: String, partial: &Path) -> String {
    let retry_note = if partial.is_file() {
        " The partial download was kept so TypeSpeak can resume it."
    } else {
        " TypeSpeak removed the invalid partial file; retry the download."
    };
    format!("{error}{retry_note}")
}

fn finish_install(partial: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        fs::remove_file(destination)
            .map_err(|error| format!("Could not replace the incomplete model: {error}"))?;
    }
    fs::rename(partial, destination)
        .map_err(|error| format!("Could not finish installing the model: {error}"))
}

fn download_to_partial(
    app: &AppHandle,
    spec: DownloadSpec<'_>,
    partial: &Path,
) -> Result<(), String> {
    let partial_file = PartialFile {
        path: partial,
        bytes: resumable_bytes(partial, spec.expected_bytes),
    };
    let response = recover_or_restart_download(
        app,
        spec,
        partial_file,
        send_download_request(spec.url, partial_file.bytes)?,
    )?;
    let Some(response) = response else {
        return Ok(());
    };
    let prepared = prepare_download(response, partial_file.bytes, spec.expected_bytes)?;
    stream_download(app, spec, partial, prepared)
}

fn recover_or_restart_download(
    app: &AppHandle,
    spec: DownloadSpec<'_>,
    partial: PartialFile<'_>,
    response: Response,
) -> Result<Option<Response>, String> {
    if response.status() != StatusCode::RANGE_NOT_SATISFIABLE {
        return Ok(Some(response));
    }
    let can_verify = spec.expected_bytes == Some(partial.bytes)
        || (spec.expected_bytes.is_none() && spec.expected_sha256.is_some());
    if can_verify {
        verify_existing_partial(app, spec, partial.path, partial.bytes)?;
        return Ok(None);
    }
    fs::remove_file(partial.path)
        .map_err(|error| format!("Could not restart the partial model download: {error}"))?;
    send_download_request(spec.url, 0).map(Some)
}

fn verify_existing_partial(
    app: &AppHandle,
    spec: DownloadSpec<'_>,
    partial: &Path,
    downloaded: u64,
) -> Result<(), String> {
    let mut hasher = Sha256::new();
    hash_file(partial, &mut hasher)?;
    verify_transfer(app, spec, partial, downloaded, hasher)
}

fn download_client() -> Result<Client, String> {
    Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(60 * 60 * 3))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|error| format!("Could not prepare the model download: {error}"))
}

fn send_download_request(url: &str, partial_bytes: u64) -> Result<Response, String> {
    let mut request = download_client()?.get(normalize_download_url(url)?);
    if partial_bytes > 0 {
        request = request.header(RANGE, format!("bytes={partial_bytes}-"));
    }
    request
        .send()
        .map_err(|error| format!("Could not download the model: {error}"))
}

fn prepare_download(
    response: Response,
    partial_bytes: u64,
    expected_bytes: Option<u64>,
) -> Result<PreparedDownload, String> {
    let response = response
        .error_for_status()
        .map_err(|error| format!("The model host refused the download: {error}"))?;
    let resuming = partial_bytes > 0 && response.status() == StatusCode::PARTIAL_CONTENT;
    let starting_bytes = if resuming { partial_bytes } else { 0 };
    verify_reported_size(&response, starting_bytes, expected_bytes)?;
    Ok(PreparedDownload {
        total_bytes: expected_bytes.or_else(|| {
            response
                .content_length()
                .map(|remaining| starting_bytes.saturating_add(remaining))
        }),
        response,
        starting_bytes,
        resuming,
    })
}

fn verify_reported_size(
    response: &Response,
    starting_bytes: u64,
    expected_bytes: Option<u64>,
) -> Result<(), String> {
    let (Some(expected), Some(remaining)) = (expected_bytes, response.content_length()) else {
        return Ok(());
    };
    let reported = starting_bytes.saturating_add(remaining);
    if reported == expected {
        return Ok(());
    }
    Err(format!(
        "The model host reported an unexpected size ({reported} bytes; expected {expected})."
    ))
}

fn stream_download(
    app: &AppHandle,
    spec: DownloadSpec<'_>,
    partial: &Path,
    mut prepared: PreparedDownload,
) -> Result<(), String> {
    let mut transfer = TransferState::new(partial, &prepared)?;
    emit_progress(
        app,
        spec.id,
        "downloading",
        transfer.downloaded_bytes,
        transfer.total_bytes,
    );
    copy_download(app, spec.id, &mut prepared.response, &mut transfer)?;
    let (downloaded, hasher) = transfer.finish()?;
    verify_transfer(app, spec, partial, downloaded, hasher)
}

fn copy_download(
    app: &AppHandle,
    model_id: &str,
    response: &mut Response,
    transfer: &mut TransferState,
) -> Result<(), String> {
    let mut buffer = vec![0_u8; DOWNLOAD_CHUNK_BYTES];
    loop {
        let read = response
            .read(&mut buffer)
            .map_err(|error| format!("The model download was interrupted: {error}"))?;
        if read == 0 {
            break;
        }
        transfer.append(app, model_id, &buffer[..read])?;
    }
    Ok(())
}

fn verify_transfer(
    app: &AppHandle,
    spec: DownloadSpec<'_>,
    partial: &Path,
    downloaded: u64,
    hasher: Sha256,
) -> Result<(), String> {
    if spec.expected_bytes.is_some() || spec.expected_sha256.is_some() {
        emit_progress(app, spec.id, "verifying", downloaded, spec.expected_bytes);
    }
    let verification = verify_download_hash(
        downloaded,
        spec.expected_bytes,
        spec.expected_sha256,
        hasher,
    );
    if verification.is_err() && invalid_partial_is_complete(spec, downloaded) {
        let _ = fs::remove_file(partial);
    }
    verification
}

fn invalid_partial_is_complete(spec: DownloadSpec<'_>, downloaded: u64) -> bool {
    spec.expected_bytes
        .is_some_and(|expected| downloaded >= expected)
        || spec.expected_sha256.is_some()
}

fn resumable_bytes(path: &Path, expected_bytes: Option<u64>) -> u64 {
    let bytes = path.metadata().map(|metadata| metadata.len()).unwrap_or(0);
    if expected_bytes.is_some_and(|expected| bytes > expected) {
        let _ = fs::remove_file(path);
        0
    } else {
        bytes
    }
}

impl TransferState {
    fn new(path: &Path, prepared: &PreparedDownload) -> Result<Self, String> {
        let mut hasher = Sha256::new();
        let file = if prepared.resuming {
            hash_file(path, &mut hasher)?;
            resume_partial(path)?
        } else {
            create_partial(path)?
        };
        Ok(Self {
            writer: BufWriter::new(file),
            hasher,
            downloaded_bytes: prepared.starting_bytes,
            total_bytes: prepared.total_bytes,
            last_event: Instant::now() - Duration::from_secs(1),
        })
    }

    fn append(&mut self, app: &AppHandle, model_id: &str, bytes: &[u8]) -> Result<(), String> {
        self.writer
            .write_all(bytes)
            .map_err(|error| format!("Could not save the downloaded model: {error}"))?;
        self.hasher.update(bytes);
        self.downloaded_bytes += bytes.len() as u64;
        self.report_progress_if_due(app, model_id);
        Ok(())
    }

    fn report_progress_if_due(&mut self, app: &AppHandle, model_id: &str) {
        if self.last_event.elapsed() < Duration::from_millis(180) {
            return;
        }
        emit_progress(
            app,
            model_id,
            "downloading",
            self.downloaded_bytes,
            self.total_bytes,
        );
        self.last_event = Instant::now();
    }

    fn finish(mut self) -> Result<(u64, Sha256), String> {
        self.writer
            .flush()
            .map_err(|error| format!("Could not finish writing the model: {error}"))?;
        self.writer
            .get_ref()
            .sync_all()
            .map_err(|error| format!("Could not sync the downloaded model to disk: {error}"))?;
        Ok((self.downloaded_bytes, self.hasher))
    }
}

fn create_partial(path: &Path) -> Result<File, String> {
    File::create(path)
        .map_err(|error| format!("Could not create the temporary model file: {error}"))
}

fn resume_partial(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|error| format!("Could not resume the temporary model file: {error}"))
}

fn hash_file(path: &Path, hasher: &mut Sha256) -> Result<(), String> {
    let mut file =
        File::open(path).map_err(|error| format!("Could not verify the partial model: {error}"))?;
    let mut buffer = vec![0_u8; DOWNLOAD_CHUNK_BYTES];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("Could not read the partial model: {error}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(())
}

fn verify_download_hash(
    downloaded: u64,
    expected_bytes: Option<u64>,
    expected_sha256: Option<&str>,
    hasher: Sha256,
) -> Result<(), String> {
    if let Some(expected) = expected_bytes {
        if downloaded != expected {
            return Err(format!(
                "The model download is incomplete ({downloaded} of {expected} bytes)."
            ));
        }
    }
    if let Some(expected) = expected_sha256 {
        let actual = format!("{:x}", hasher.finalize());
        if !actual.eq_ignore_ascii_case(expected) {
            return Err("The downloaded model failed its SHA-256 integrity check.".into());
        }
    }
    Ok(())
}

fn emit_progress(
    app: &AppHandle,
    model_id: &str,
    stage: &'static str,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
) {
    let percent = total_bytes
        .filter(|total| *total > 0)
        .map(|total| ((downloaded_bytes.saturating_mul(100) / total).min(100)) as u8);
    let _ = app.emit(
        DOWNLOAD_EVENT,
        DownloadProgress {
            id: model_id.to_owned(),
            stage,
            downloaded_bytes,
            total_bytes,
            percent,
        },
    );
}

fn normalize_download_url(value: &str) -> Result<Url, String> {
    let mut url = Url::parse(value).map_err(|error| format!("Invalid model URL: {error}"))?;
    if url.scheme() != "https" {
        return Err("Managed model downloads must use HTTPS.".into());
    }
    let host = url
        .host_str()
        .ok_or_else(|| "The model URL has no host.".to_string())?;
    if matches!(host, "localhost" | "127.0.0.1" | "::1") {
        return Err("Managed model downloads cannot target this computer.".into());
    }
    if host.eq_ignore_ascii_case("huggingface.co") && url.path().contains("/blob/") {
        let normalized_path = url.path().replacen("/blob/", "/resolve/", 1);
        url.set_path(&normalized_path);
    }
    Ok(url)
}

fn validate_model_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 80
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_".contains(character))
    {
        return Err("The managed model ID contains unsupported characters.".into());
    }
    Ok(())
}

fn validate_model_file_name(value: &str) -> Result<String, String> {
    let file_name = Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Enter a valid GGUF file name.".to_string())?;
    if file_name != value
        || file_name.len() > 160
        || !file_name.to_ascii_lowercase().ends_with(".gguf")
        || !file_name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
    {
        return Err("Managed model files must be a simple .gguf file name.".into());
    }
    Ok(file_name.to_owned())
}

fn validate_sha256(value: &str) -> Result<String, String> {
    if value.len() != 64 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err("SHA-256 must contain exactly 64 hexadecimal characters.".into());
    }
    Ok(value.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::{catalog_entry, normalize_download_url, validate_model_file_name, validate_sha256};

    #[test]
    fn catalog_contains_every_visible_managed_model() {
        for id in [
            "cohere-local",
            "qwen-local",
            "omni-local",
            "translator-m2m100",
        ] {
            assert!(catalog_entry(id).is_some());
        }
    }

    #[test]
    fn custom_model_files_cannot_escape_the_managed_directory() {
        assert!(validate_model_file_name("my-arabic-model.gguf").is_ok());
        assert!(validate_model_file_name("..\\model.gguf").is_err());
        assert!(validate_model_file_name("../model.gguf").is_err());
        assert!(validate_model_file_name("model.bin").is_err());
    }

    #[test]
    fn managed_downloads_require_remote_https() {
        assert!(
            normalize_download_url("https://huggingface.co/org/model/resolve/main/a.gguf").is_ok()
        );
        assert!(normalize_download_url("http://example.com/model.gguf").is_err());
        assert!(normalize_download_url("https://localhost/model.gguf").is_err());
    }

    #[test]
    fn hugging_face_blob_links_become_download_links() {
        let url = normalize_download_url(
            "https://huggingface.co/org/model/blob/main/model.gguf?download=true",
        )
        .unwrap();
        assert_eq!(url.path(), "/org/model/resolve/main/model.gguf");
        assert_eq!(url.query(), Some("download=true"));
    }

    #[test]
    fn checksum_validation_is_strict() {
        assert!(validate_sha256(&"a".repeat(64)).is_ok());
        assert!(validate_sha256("not-a-checksum").is_err());
    }
}
