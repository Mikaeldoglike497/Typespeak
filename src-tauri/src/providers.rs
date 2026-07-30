use crate::model_manager;
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::blocking::{multipart, Client};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const WHISPER_ENGINE: &str = "whisper";
const WHISPER_MODEL_ID: &str = "whisper-local";
const COHERE_ENGINE: &str = "cohere";
const QWEN_ENGINE: &str = "qwen3";
const OMNI_ENGINE: &str = "omniasr";
const TRANSLATOR_MODEL_ID: &str = "translator-m2m100";
const DEFAULT_MODEL_FILE: &str = "ggml-large-v3-turbo-q5_0.bin";
const MODEL_CANDIDATES: [&str; 4] = [
    DEFAULT_MODEL_FILE,
    "ggml-large-v3-q5_0.bin",
    "ggml-medium-q5_0.bin",
    "ggml-small-q5_1.bin",
];
const MINIMUM_MODEL_BYTES: u64 = 20 * 1024 * 1024;
const MIXED_SILENCE_MS: usize = 280;
const MIXED_MIN_CHUNK_MS: usize = 1_400;
const MIXED_MAX_CHUNKS: usize = 8;
static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionRequest {
    pub engine: String,
    pub audio_base64: String,
    pub language: String,
    pub duration_ms: u64,
    #[serde(default)]
    pub glossary: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointTranscriptionRequest {
    pub connection_id: String,
    pub connection: EndpointConnection,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub model: String,
    pub audio_base64: String,
    pub language: String,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomTranscriptionRequest {
    pub connection_id: String,
    pub file_name: String,
    pub backend: String,
    pub audio_base64: String,
    pub language: String,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EndpointConnection {
    Local,
    Api,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionResult {
    pub engine: String,
    pub model: String,
    pub text: String,
    pub elapsed_ms: u128,
    pub audio_duration_ms: u64,
    pub ok: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub id: &'static str,
    pub name: &'static str,
    pub model: String,
    pub configured: bool,
    pub setup_hint: String,
    pub note: &'static str,
    pub managed: bool,
    pub download_bytes: Option<u64>,
}

#[derive(Clone, Debug)]
struct WhisperInstallation {
    executable: PathBuf,
    servers: Vec<PathBuf>,
    model: PathBuf,
}

#[derive(Clone, Default)]
pub struct WhisperRuntime {
    active_server: Arc<Mutex<Option<WarmWhisperServer>>>,
}

struct WarmWhisperServer {
    child: Child,
    model: PathBuf,
    port: u16,
}

impl WhisperRuntime {
    fn transcribe(&self, job: &WhisperJob<'_>) -> Result<String, String> {
        let mut active_server = self
            .active_server
            .lock()
            .map_err(|_| "The local Whisper service is unavailable.".to_string())?;
        ensure_warm_server(&mut active_server, job.installation)?;
        active_server
            .as_ref()
            .ok_or_else(|| "The local Whisper service did not start.".to_string())?
            .transcribe(job)
    }

    pub fn shutdown(&self) {
        if let Ok(mut active_server) = self.active_server.lock() {
            *active_server = None;
        }
    }
}

impl WarmWhisperServer {
    fn launch(installation: &WhisperInstallation) -> Result<Self, String> {
        let mut launch_errors = Vec::new();
        for server_executable in &installation.servers {
            match launch_whisper_server(server_executable, &installation.model) {
                Ok(server) => return Ok(server),
                Err(error) => {
                    launch_errors.push(format!("{}: {error}", server_executable.display()))
                }
            }
        }
        Err(format!(
            "No bundled Whisper service could start. {}",
            launch_errors.join(" | ")
        ))
    }

    fn matches_running_model(&mut self, model: &Path) -> Result<bool, String> {
        if self.model.as_path() != model {
            return Ok(false);
        }
        self.child
            .try_wait()
            .map(|status| status.is_none())
            .map_err(|error| format!("Could not check the warm Whisper service: {error}"))
    }

    fn transcribe(&self, job: &WhisperJob<'_>) -> Result<String, String> {
        let response = whisper_server_client(job.duration_ms)?
            .post(format!("http://127.0.0.1:{}/inference", self.port))
            .multipart(whisper_server_form(job)?)
            .send()
            .map_err(|error| format!("The warm Whisper service could not transcribe: {error}"))?;
        whisper_server_text(response)
    }
}

fn launch_whisper_server(executable: &Path, model: &Path) -> Result<WarmWhisperServer, String> {
    let port = available_local_port()?;
    let mut child = whisper_server_command(executable, model, port)
        .spawn()
        .map_err(|error| format!("could not start ({error})"))?;
    if let Err(error) = wait_for_whisper_server(&mut child, port) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    Ok(WarmWhisperServer {
        child,
        model: model.to_owned(),
        port,
    })
}

impl Drop for WarmWhisperServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn ensure_warm_server(
    active_server: &mut Option<WarmWhisperServer>,
    installation: &WhisperInstallation,
) -> Result<(), String> {
    let server_is_reusable = match active_server.as_mut() {
        Some(server) => match server.matches_running_model(&installation.model) {
            Ok(is_reusable) => is_reusable,
            Err(error) => {
                *active_server = None;
                return Err(error);
            }
        },
        None => false,
    };
    if server_is_reusable {
        return Ok(());
    }
    *active_server = Some(WarmWhisperServer::launch(installation)?);
    Ok(())
}

fn whisper_server_command(executable: &Path, model: &Path, port: u16) -> Command {
    let mut command = Command::new(executable);
    command
        .arg("-m")
        .arg(model)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .arg("-t")
        .arg(inference_threads().to_string())
        .arg("-nt")
        .arg("--no-language-probabilities")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    hide_child_console(&mut command);
    command
}

#[cfg(windows)]
fn hide_child_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_child_console(_command: &mut Command) {}

fn available_local_port() -> Result<u16, String> {
    TcpListener::bind(("127.0.0.1", 0))
        .and_then(|listener| listener.local_addr())
        .map(|address| address.port())
        .map_err(|error| format!("Could not reserve a local port for Whisper: {error}"))
}

fn wait_for_whisper_server(child: &mut Child, port: u16) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(90);
    while Instant::now() < deadline {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("Could not monitor the warm Whisper service: {error}"))?
        {
            return Err(format!(
                "The warm Whisper service stopped during startup: {status}"
            ));
        }
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(40));
    }
    Err("The warm Whisper service took too long to load the model.".into())
}

fn whisper_server_client(duration_ms: u64) -> Result<Client, String> {
    Client::builder()
        .connect_timeout(Duration::from_secs(2))
        .timeout(inference_timeout(duration_ms))
        .build()
        .map_err(|error| format!("Could not prepare the local Whisper connection: {error}"))
}

fn whisper_server_form(job: &WhisperJob<'_>) -> Result<multipart::Form, String> {
    let audio = multipart::Part::bytes(job.audio.to_vec())
        .file_name("recording.wav")
        .mime_str("audio/wav")
        .map_err(|error| format!("Could not prepare the local audio upload: {error}"))?;
    Ok(multipart::Form::new()
        .part("file", audio)
        .text("language", whisper_language(job.language).to_owned())
        .text("prompt", glossary_prompt(job.glossary, job.language))
        .text("response_format", "json")
        .text("translate", "false")
        .text("no_timestamps", "true")
        .text("token_timestamps", "false"))
}

fn whisper_server_text(response: reqwest::blocking::Response) -> Result<String, String> {
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| format!("Could not read the warm Whisper response: {error}"))?;
    if !status.is_success() {
        return Err(format!(
            "The warm Whisper service returned {status}: {}",
            body.chars().take(500).collect::<String>()
        ));
    }
    endpoint_text(&body)
}

#[derive(Clone, Debug)]
struct CrispInstallation {
    executable: PathBuf,
    model: PathBuf,
    backend: String,
}

struct WhisperJob<'a> {
    installation: &'a WhisperInstallation,
    audio: &'a [u8],
    language: &'a str,
    glossary: &'a [String],
    duration_ms: u64,
}

struct ResultContext {
    engine: String,
    model: String,
    duration_ms: u64,
    started: Instant,
}

impl ResultContext {
    fn new(engine: &str, model: &str, duration_ms: u64, started: Instant) -> Self {
        Self {
            engine: engine.into(),
            model: model.into(),
            duration_ms,
            started,
        }
    }

    fn success(self, text: String) -> TranscriptionResult {
        TranscriptionResult {
            engine: self.engine,
            model: self.model,
            text,
            elapsed_ms: self.started.elapsed().as_millis(),
            audio_duration_ms: self.duration_ms,
            ok: true,
            error: None,
        }
    }

    fn failure(self, error: String) -> TranscriptionResult {
        TranscriptionResult {
            engine: self.engine,
            model: self.model,
            text: String::new(),
            elapsed_ms: self.started.elapsed().as_millis(),
            audio_duration_ms: self.duration_ms,
            ok: false,
            error: Some(error),
        }
    }
}

struct SessionFiles {
    audio: PathBuf,
    transcript: PathBuf,
    output_prefix: PathBuf,
}

impl SessionFiles {
    fn create() -> Result<Self, String> {
        let session_root = env::temp_dir().join("typespeak");
        fs::create_dir_all(&session_root)
            .map_err(|error| format!("Could not prepare local audio processing: {error}"))?;
        let sequence = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let stem = format!("utterance-{}-{sequence}", std::process::id());
        let output_prefix = session_root.join(&stem);
        Ok(Self {
            audio: session_root.join(format!("{stem}.wav")),
            transcript: session_root.join(format!("{stem}.txt")),
            output_prefix,
        })
    }
}

impl Drop for SessionFiles {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.audio);
        let _ = fs::remove_file(&self.transcript);
    }
}

pub fn statuses() -> Vec<EngineStatus> {
    vec![
        whisper_status(),
        managed_asr_status(
            COHERE_ENGINE,
            "Cohere Transcribe Arabic",
            "cohere-local",
            "Best Arabic and Arabic-English accuracy. Runs locally through CrispASR.",
        ),
        managed_asr_status(
            QWEN_ENGINE,
            "Qwen3-ASR 0.6B",
            "qwen-local",
            "Light multilingual speech model with automatic language detection.",
        ),
        managed_asr_status(
            OMNI_ENGINE,
            "OmniASR CTC 300M",
            "omni-local",
            "Compact multilingual CTC model for lower-memory computers.",
        ),
    ]
}

fn whisper_status() -> EngineStatus {
    let catalog = model_manager::catalog_entry(WHISPER_MODEL_ID).expect("built-in catalog entry");
    let installation = find_whisper_installation();
    let model = installation
        .as_ref()
        .map(|paths| model_name(&paths.model))
        .unwrap_or(catalog.file_name)
        .to_owned();
    EngineStatus {
        id: WHISPER_ENGINE,
        name: "Whisper local",
        model,
        configured: installation.is_some(),
        setup_hint: if installation.is_some() {
            "Installed and ready".into()
        } else {
            format!("Download {}", format_bytes(catalog.bytes))
        },
        note: "Runs fully on this computer through a warm whisper.cpp service. Uses CUDA automatically when installed; no audio is uploaded.",
        managed: true,
        download_bytes: Some(catalog.bytes),
    }
}

fn managed_asr_status(
    engine_id: &'static str,
    name: &'static str,
    catalog_id: &str,
    note: &'static str,
) -> EngineStatus {
    let catalog = model_manager::catalog_entry(catalog_id).expect("built-in catalog entry");
    let model_status = model_manager::catalog_status(catalog_id).expect("built-in model status");
    let runtime_ready = find_crisp_executable().is_some();
    EngineStatus {
        id: engine_id,
        name,
        model: catalog.file_name.into(),
        configured: runtime_ready && model_status.installed,
        setup_hint: if model_status.installed {
            "Installed and ready".into()
        } else {
            format!("Download {}", format_bytes(catalog.bytes))
        },
        note,
        managed: true,
        download_bytes: Some(catalog.bytes),
    }
}

pub fn transcribe(
    request: TranscriptionRequest,
    whisper_runtime: &WhisperRuntime,
) -> TranscriptionResult {
    let started = Instant::now();
    let engine = request.engine.to_lowercase();
    let context = ResultContext::new(&engine, model_for(&engine), request.duration_ms, started);
    let audio = match decode_audio(&request.audio_base64) {
        Ok(audio) => audio,
        Err(error) => return context.failure(error),
    };

    if audio.len() < 1_000 {
        return context.failure(
            "The recording is too short. Hold the shortcut and speak for at least one second."
                .into(),
        );
    }

    match engine.as_str() {
        WHISPER_ENGINE => transcribe_with_whisper(audio, request, started, whisper_runtime),
        COHERE_ENGINE | QWEN_ENGINE | OMNI_ENGINE => {
            transcribe_with_managed_crisp(audio, request, started)
        }
        _ => context.failure(format!("Unknown local engine: {}", request.engine)),
    }
}

pub fn transcribe_endpoint(request: EndpointTranscriptionRequest) -> TranscriptionResult {
    let started = Instant::now();
    let context = ResultContext::new(
        &request.connection_id,
        &request.model,
        request.duration_ms,
        started,
    );
    match endpoint_transcript(&request) {
        Ok(text) => context.success(text),
        Err(error) => context.failure(error),
    }
}

pub fn transcribe_custom_model(request: CustomTranscriptionRequest) -> TranscriptionResult {
    let started = Instant::now();
    let context = ResultContext::new(
        &request.connection_id,
        &request.file_name,
        request.duration_ms,
        started,
    );
    let audio = match decode_audio(&request.audio_base64) {
        Ok(audio) => audio,
        Err(error) => return context.failure(error),
    };
    if audio.len() < 1_000 {
        return context.failure(
            "The recording is too short. Hold the shortcut and speak for at least one second."
                .into(),
        );
    }
    let model = match model_manager::custom_model_path_by_name(&request.file_name) {
        Ok(model) if model.is_file() => model,
        Ok(_) => {
            return context.failure(
                "This managed model is not installed. Download it from Settings first.".into(),
            )
        }
        Err(error) => return context.failure(error),
    };
    let executable = match find_crisp_executable() {
        Some(executable) => executable,
        None => return context.failure(crisp_runtime_error()),
    };
    let backend = match validate_backend(&request.backend) {
        Ok(backend) => backend,
        Err(error) => return context.failure(error),
    };
    let installation = CrispInstallation {
        executable,
        model,
        backend,
    };
    match run_crisp(
        &installation,
        &audio,
        &request.language,
        request.duration_ms,
    ) {
        Ok(text) => context.success(text),
        Err(error) => context.failure(error),
    }
}

fn endpoint_transcript(request: &EndpointTranscriptionRequest) -> Result<String, String> {
    let audio = decode_audio(&request.audio_base64)?;
    if audio.len() < 1_000 {
        return Err(
            "The recording is too short. Hold the shortcut and speak for at least one second."
                .into(),
        );
    }
    validate_endpoint(&request.endpoint, &request.connection)?;
    let form = endpoint_form(audio, &request.model, &request.language)?;
    let client = endpoint_client(request.duration_ms)?;
    let body = send_endpoint_request(client, request, form)?;
    endpoint_text(&body)
}

fn endpoint_form(audio: Vec<u8>, model: &str, language: &str) -> Result<multipart::Form, String> {
    let mut form = multipart::Form::new().text("model", model.to_owned());
    if let Some(language) = endpoint_language(language) {
        form = form.text("language", language.to_owned());
    }
    let audio_part = multipart::Part::bytes(audio)
        .file_name("typespeak.wav")
        .mime_str("audio/wav")
        .map_err(|error| format!("Could not prepare audio upload: {error}"))?;
    Ok(form.part("file", audio_part))
}

fn endpoint_client(duration_ms: u64) -> Result<Client, String> {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(inference_timeout(duration_ms))
        .build()
        .map_err(|error| format!("Could not prepare endpoint client: {error}"))
}

fn send_endpoint_request(
    client: Client,
    request: &EndpointTranscriptionRequest,
    form: multipart::Form,
) -> Result<String, String> {
    let mut call = client.post(&request.endpoint).multipart(form);
    if let Some(api_key) = request
        .api_key
        .as_deref()
        .filter(|key| !key.trim().is_empty())
    {
        call = call.bearer_auth(api_key);
    }
    let response = call.send().map_err(|error| {
        format!(
            "Could not reach the model endpoint. Check that it is running and the URL is correct: {error}"
        )
    })?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| format!("Could not read endpoint response: {error}"))?;
    if !status.is_success() {
        let detail: String = body.chars().take(500).collect();
        return Err(format!("Model endpoint returned {status}: {detail}"));
    }
    Ok(body)
}

fn validate_endpoint(endpoint: &str, connection: &EndpointConnection) -> Result<(), String> {
    let url = Url::parse(endpoint).map_err(|error| format!("Invalid endpoint URL: {error}"))?;
    match connection {
        EndpointConnection::Local if is_loopback_url(&url) => Ok(()),
        EndpointConnection::Local => {
            Err("Local model endpoints must use localhost, 127.0.0.1, or ::1.".into())
        }
        EndpointConnection::Api if url.scheme() == "https" => Ok(()),
        EndpointConnection::Api => Err("Cloud API endpoints must use HTTPS.".into()),
    }
}

fn is_loopback_url(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https")
        && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"))
}

fn endpoint_language(language: &str) -> Option<&str> {
    match language {
        "ar" => Some("ar"),
        "en" => Some("en"),
        _ => None,
    }
}

fn endpoint_text(body: &str) -> Result<String, String> {
    let payload: Value = serde_json::from_str(body)
        .map_err(|error| format!("Endpoint returned invalid JSON: {error}"))?;
    let text = payload
        .get("text")
        .and_then(Value::as_str)
        .or_else(|| {
            payload
                .get("data")
                .and_then(|data| data.get("text"))
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| "Endpoint response did not contain a non-empty `text` field.".to_string())?;
    Ok(text.to_owned())
}

fn transcribe_with_whisper(
    audio: Vec<u8>,
    request: TranscriptionRequest,
    started: Instant,
    whisper_runtime: &WhisperRuntime,
) -> TranscriptionResult {
    let installation = match require_whisper_installation() {
        Ok(installation) => installation,
        Err(error) => {
            return ResultContext::new(
                WHISPER_ENGINE,
                DEFAULT_MODEL_FILE,
                request.duration_ms,
                started,
            )
            .failure(error)
        }
    };
    let context = ResultContext::new(
        WHISPER_ENGINE,
        model_name(&installation.model),
        request.duration_ms,
        started,
    );
    let job = WhisperJob {
        installation: &installation,
        audio: &audio,
        language: &request.language,
        glossary: &request.glossary,
        duration_ms: request.duration_ms,
    };
    match run_whisper(&job, whisper_runtime) {
        Ok(text) => context.success(text),
        Err(error) => context.failure(error),
    }
}

fn transcribe_with_managed_crisp(
    audio: Vec<u8>,
    request: TranscriptionRequest,
    started: Instant,
) -> TranscriptionResult {
    let engine = request.engine.to_lowercase();
    let catalog_id = match engine.as_str() {
        COHERE_ENGINE => "cohere-local",
        QWEN_ENGINE => "qwen-local",
        OMNI_ENGINE => "omni-local",
        _ => {
            return ResultContext::new(&engine, "unknown", request.duration_ms, started)
                .failure(format!("Unknown managed engine: {}", request.engine))
        }
    };
    let installation = match require_crisp_installation(catalog_id, &engine) {
        Ok(installation) => installation,
        Err(error) => {
            return ResultContext::new(&engine, model_for(&engine), request.duration_ms, started)
                .failure(error)
        }
    };
    let context = ResultContext::new(
        &engine,
        model_name(&installation.model),
        request.duration_ms,
        started,
    );
    match run_crisp(
        &installation,
        &audio,
        &request.language,
        request.duration_ms,
    ) {
        Ok(text) => context.success(text),
        Err(error) => context.failure(error),
    }
}

fn decode_audio(audio_base64: &str) -> Result<Vec<u8>, String> {
    STANDARD
        .decode(audio_base64.as_bytes())
        .map_err(|error| format!("The recorded audio could not be decoded: {error}"))
}

fn run_crisp(
    installation: &CrispInstallation,
    audio: &[u8],
    language: &str,
    duration_ms: u64,
) -> Result<String, String> {
    let files = SessionFiles::create()?;
    fs::write(&files.audio, audio)
        .map_err(|error| format!("Could not prepare the local WAV file: {error}"))?;
    let mut command = Command::new(&installation.executable);
    if !installation.backend.is_empty() {
        command.arg("--backend").arg(&installation.backend);
    }
    command
        .arg("-m")
        .arg(&installation.model)
        .arg("-f")
        .arg(&files.audio)
        .arg("-l")
        .arg(crisp_language(&installation.backend, language))
        .arg("-t")
        .arg(inference_threads().to_string())
        .arg("-nt")
        .arg("-np")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_child_console(&mut command);
    let child = command
        .spawn()
        .map_err(|error| format!("Could not start the local CrispASR engine: {error}"))?;
    let output = wait_for_output(child, inference_timeout(duration_ms))?;
    if !output.status.success() {
        return Err(crisp_command_error(&installation.backend, &output));
    }
    let transcript = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if transcript.is_empty() {
        Err("The local model returned an empty transcript.".into())
    } else {
        Ok(transcript)
    }
}

fn run_whisper(job: &WhisperJob<'_>, runtime: &WhisperRuntime) -> Result<String, String> {
    if job.language == "mixed" {
        if let Some(chunks) = mixed_wav_chunks(job.audio) {
            let mut transcripts = Vec::with_capacity(chunks.len());
            for (audio, duration_ms) in &chunks {
                let chunk_job = WhisperJob {
                    installation: job.installation,
                    audio,
                    language: job.language,
                    glossary: job.glossary,
                    duration_ms: *duration_ms,
                };
                transcripts.push(run_whisper_once(&chunk_job, runtime)?);
            }
            return Ok(join_mixed_transcripts(transcripts));
        }
    }
    run_whisper_once(job, runtime)
}

fn run_whisper_once(job: &WhisperJob<'_>, runtime: &WhisperRuntime) -> Result<String, String> {
    if !job.installation.servers.is_empty() {
        runtime.transcribe(job)
    } else {
        run_whisper_cli(job)
    }
}

fn join_mixed_transcripts(transcripts: Vec<String>) -> String {
    transcripts
        .into_iter()
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn mixed_wav_chunks(audio: &[u8]) -> Option<Vec<(Vec<u8>, u64)>> {
    let wav = Pcm16MonoWav::parse(audio)?;
    let boundaries = mixed_silence_boundaries(&wav.samples, wav.sample_rate);
    if boundaries.is_empty() {
        return None;
    }

    let mut chunks = Vec::with_capacity(boundaries.len() + 1);
    let mut start = 0;
    for end in boundaries
        .into_iter()
        .chain(std::iter::once(wav.samples.len()))
        .take(MIXED_MAX_CHUNKS)
    {
        if end <= start {
            continue;
        }
        let samples = &wav.samples[start..end];
        let duration_ms = (samples.len() as u64).saturating_mul(1_000) / u64::from(wav.sample_rate);
        chunks.push((encode_pcm16_mono_wav(samples, wav.sample_rate), duration_ms));
        start = end;
    }
    (chunks.len() > 1).then_some(chunks)
}

struct Pcm16MonoWav {
    sample_rate: u32,
    samples: Vec<i16>,
}

impl Pcm16MonoWav {
    fn parse(audio: &[u8]) -> Option<Self> {
        if audio.len() < 44 || audio.get(0..4)? != b"RIFF" || audio.get(8..12)? != b"WAVE" {
            return None;
        }
        let mut cursor: usize = 12;
        let mut format = None;
        let mut data = None;
        while cursor.checked_add(8)? <= audio.len() {
            let id = audio.get(cursor..cursor + 4)?;
            let size = read_wav_u32(audio, cursor + 4)? as usize;
            let body = cursor.checked_add(8)?;
            let end = body.checked_add(size)?;
            if end > audio.len() {
                return None;
            }
            if id == b"fmt " && size >= 16 {
                format = Some((
                    read_wav_u16(audio, body)?,
                    read_wav_u16(audio, body + 2)?,
                    read_wav_u32(audio, body + 4)?,
                    read_wav_u16(audio, body + 14)?,
                ));
            } else if id == b"data" {
                data = Some(audio.get(body..end)?);
            }
            cursor = end.checked_add(size % 2)?;
        }
        let (encoding, channels, sample_rate, bits_per_sample) = format?;
        let data = data?;
        if encoding != 1 || channels != 1 || sample_rate == 0 || bits_per_sample != 16 {
            return None;
        }
        let samples = data
            .chunks_exact(2)
            .map(|bytes| i16::from_le_bytes([bytes[0], bytes[1]]))
            .collect::<Vec<_>>();
        (!samples.is_empty()).then_some(Self {
            sample_rate,
            samples,
        })
    }
}

fn read_wav_u16(audio: &[u8], offset: usize) -> Option<u16> {
    let bytes: [u8; 2] = audio.get(offset..offset.checked_add(2)?)?.try_into().ok()?;
    Some(u16::from_le_bytes(bytes))
}

fn read_wav_u32(audio: &[u8], offset: usize) -> Option<u32> {
    let bytes: [u8; 4] = audio.get(offset..offset.checked_add(4)?)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn mixed_silence_boundaries(samples: &[i16], sample_rate: u32) -> Vec<usize> {
    let window_samples = (sample_rate as usize / 50).max(1);
    let levels = samples
        .chunks(window_samples)
        .map(window_rms)
        .collect::<Vec<_>>();
    if levels.len() < 3 {
        return Vec::new();
    }

    let mut sorted_levels = levels.clone();
    sorted_levels.sort_by(|left, right| left.total_cmp(right));
    let reference_index = (sorted_levels.len() * 9 / 10).min(sorted_levels.len() - 1);
    let speech_reference = sorted_levels[reference_index];
    let silence_threshold = (speech_reference * 0.12).max(180.0);
    let minimum_silence_windows = (MIXED_SILENCE_MS / 20).max(1);
    let minimum_chunk_samples = (sample_rate as usize).saturating_mul(MIXED_MIN_CHUNK_MS) / 1_000;

    let mut boundaries = Vec::new();
    let mut silence_start = None;
    let mut last_boundary = 0;
    for (index, level) in levels
        .iter()
        .copied()
        .chain(std::iter::once(f32::INFINITY))
        .enumerate()
    {
        if level <= silence_threshold {
            silence_start.get_or_insert(index);
            continue;
        }
        let Some(start_window) = silence_start.take() else {
            continue;
        };
        let silence_windows = index.saturating_sub(start_window);
        if silence_windows < minimum_silence_windows {
            continue;
        }
        let boundary_window = start_window + silence_windows / 2;
        let boundary = boundary_window
            .saturating_mul(window_samples)
            .min(samples.len());
        let before = boundary.saturating_sub(last_boundary);
        let after = samples.len().saturating_sub(boundary);
        if before >= minimum_chunk_samples && after >= minimum_chunk_samples {
            boundaries.push(boundary);
            last_boundary = boundary;
            if boundaries.len() + 1 >= MIXED_MAX_CHUNKS {
                break;
            }
        }
    }
    boundaries
}

fn window_rms(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let squared_sum = samples.iter().fold(0.0_f64, |sum, sample| {
        let value = f64::from(*sample);
        sum + value * value
    });
    (squared_sum / samples.len() as f64).sqrt() as f32
}

fn encode_pcm16_mono_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let data_bytes = samples.len().saturating_mul(2);
    let riff_bytes = 36_usize.saturating_add(data_bytes).min(u32::MAX as usize) as u32;
    let mut wav = Vec::with_capacity(44 + data_bytes);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_bytes.to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&sample_rate.saturating_mul(2).to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_bytes.min(u32::MAX as usize) as u32).to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

fn run_whisper_cli(job: &WhisperJob<'_>) -> Result<String, String> {
    let files = SessionFiles::create()?;
    fs::write(&files.audio, job.audio)
        .map_err(|error| format!("Could not prepare the local WAV file: {error}"))?;

    let mut command = whisper_command(job.installation, &files, job.language, job.glossary);
    let child = command
        .spawn()
        .map_err(|error| format!("Could not start the local Whisper engine: {error}"))?;
    let output = wait_for_output(child, inference_timeout(job.duration_ms))?;
    if !output.status.success() {
        return Err(command_error(&output));
    }

    let file_text = fs::read_to_string(&files.transcript).unwrap_or_default();
    let transcript = if file_text.trim().is_empty() {
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    } else {
        file_text.trim().to_owned()
    };
    if transcript.is_empty() {
        Err("The local model returned an empty transcript.".into())
    } else {
        Ok(transcript)
    }
}

fn whisper_command(
    installation: &WhisperInstallation,
    files: &SessionFiles,
    language: &str,
    glossary: &[String],
) -> Command {
    let mut command = Command::new(&installation.executable);
    command
        .arg("-m")
        .arg(&installation.model)
        .arg("-f")
        .arg(&files.audio)
        .arg("-l")
        .arg(whisper_language(language))
        .arg("-t")
        .arg(inference_threads().to_string())
        .arg("-otxt")
        .arg("-of")
        .arg(&files.output_prefix)
        .arg("-nt")
        .arg("-np")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let prompt = glossary_prompt(glossary, language);
    if !prompt.is_empty() {
        command.arg("--prompt").arg(prompt);
    }
    hide_child_console(&mut command);
    command
}

fn wait_for_output(mut child: Child, timeout: Duration) -> Result<Output, String> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|error| format!("Could not read the local model output: {error}"));
            }
            Ok(None) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(30));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "Local transcription exceeded {} seconds and was stopped.",
                    timeout.as_secs()
                ));
            }
            Err(error) => {
                let _ = child.kill();
                return Err(format!("Could not monitor the local model: {error}"));
            }
        }
    }
}

fn command_error(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if stderr.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };
    let shortened: String = detail.chars().take(500).collect();
    if shortened.is_empty() {
        format!("The local Whisper engine exited with {}.", output.status)
    } else {
        format!("Local Whisper failed: {shortened}")
    }
}

fn crisp_command_error(backend: &str, output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if stderr.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };
    let shortened: String = detail.chars().take(700).collect();
    if shortened.is_empty() {
        format!("The local {backend} engine exited with {}.", output.status)
    } else {
        format!("Local {backend} failed: {shortened}")
    }
}

fn inference_timeout(duration_ms: u64) -> Duration {
    let audio_seconds = duration_ms.div_ceil(1_000);
    Duration::from_secs((120 + audio_seconds.saturating_mul(4)).min(480))
}

fn inference_threads() -> usize {
    thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .saturating_sub(1)
        .clamp(1, 8)
}

fn whisper_language(language: &str) -> &'static str {
    match language {
        "ar" => "ar",
        "en" => "en",
        _ => "auto",
    }
}

fn crisp_language(backend: &str, language: &str) -> &'static str {
    match (backend, language) {
        (COHERE_ENGINE, "ar") => "ar",
        (COHERE_ENGINE, "en") => "en",
        (_, "ar") => "ar",
        (_, "en") => "en",
        _ => "auto",
    }
}

fn glossary_prompt(glossary: &[String], language: &str) -> String {
    let terms = glossary
        .iter()
        .map(String::as_str)
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .take(30)
        .collect::<Vec<_>>()
        .join(", ");
    let context = match language {
        "mixed" => "مرحبا، today عندي meeting مع the team. خلّينا نكمّل الـproject على TypeSpeak, وبعدين منعمل final review. English words stay in Latin script.",
        "ar" => "هيدا حديث لبناني مكتوب بالحروف العربية.",
        "en" => "Lebanese-accented English conversation.",
        _ => "Lebanese Arabic and English conversation.",
    };
    let prompt = if terms.is_empty() {
        context.to_owned()
    } else {
        format!("{context} Names and exact spellings: {terms}")
    };
    prompt.chars().take(500).collect()
}

fn require_whisper_installation() -> Result<WhisperInstallation, String> {
    find_whisper_installation().ok_or_else(|| {
        "Whisper large-v3-turbo is not installed. Select Whisper in TypeSpeak and choose Download."
            .into()
    })
}

fn require_crisp_installation(
    catalog_id: &str,
    backend: &str,
) -> Result<CrispInstallation, String> {
    let executable = find_crisp_executable().ok_or_else(crisp_runtime_error)?;
    let model = model_manager::catalog_model_path_by_id(catalog_id)
        .filter(|path| path.is_file())
        .ok_or_else(|| {
            format!(
                "{catalog_id} is not installed. Select it in TypeSpeak and confirm the one-time download."
            )
        })?;
    Ok(CrispInstallation {
        executable,
        model,
        backend: backend.to_owned(),
    })
}

fn find_crisp_executable() -> Option<PathBuf> {
    let mut candidates = env_path("TYPESPEAK_CRISPASR_EXE");
    for root in search_roots() {
        candidates.push(root.join("runtime").join("crispasr").join("crispasr.exe"));
        candidates.push(
            root.join("resources")
                .join("runtime")
                .join("crispasr")
                .join("crispasr.exe"),
        );
    }
    first_valid_file(candidates, 1_000_000)
}

fn crisp_runtime_error() -> String {
    "The bundled CrispASR runtime is missing. Reinstall TypeSpeak or restore runtime\\crispasr."
        .into()
}

fn find_whisper_installation() -> Option<WhisperInstallation> {
    let executable = first_valid_file(executable_candidates(), 1)?;
    let servers = valid_files(server_candidates(), 1);
    let model = first_valid_file(model_candidates(), MINIMUM_MODEL_BYTES)?;
    Some(WhisperInstallation {
        executable,
        servers,
        model,
    })
}

fn executable_candidates() -> Vec<PathBuf> {
    let mut candidates = env_path("TYPESPEAK_WHISPER_EXE");
    for root in search_roots() {
        candidates.push(root.join("runtime-cuda").join("whisper-cli.exe"));
        candidates.push(root.join("runtime").join("whisper-cli.exe"));
        candidates.push(root.join("whisper-cli.exe"));
        candidates.push(
            root.join("resources")
                .join("runtime-cuda")
                .join("whisper-cli.exe"),
        );
        candidates.push(
            root.join("resources")
                .join("runtime")
                .join("whisper-cli.exe"),
        );
    }
    candidates
}

fn server_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for root in search_roots() {
        candidates.push(root.join("runtime-cuda").join("whisper-server.exe"));
        candidates.push(root.join("runtime").join("whisper-server.exe"));
        candidates.push(root.join("whisper-server.exe"));
        candidates.push(
            root.join("resources")
                .join("runtime-cuda")
                .join("whisper-server.exe"),
        );
        candidates.push(
            root.join("resources")
                .join("runtime")
                .join("whisper-server.exe"),
        );
    }
    candidates
}

fn model_candidates() -> Vec<PathBuf> {
    let mut candidates = env_path("TYPESPEAK_WHISPER_MODEL");
    if let Some(managed_model) = model_manager::catalog_model_path_by_id(WHISPER_MODEL_ID) {
        candidates.push(managed_model);
    }
    for root in search_roots() {
        for model in MODEL_CANDIDATES {
            candidates.push(root.join("models").join(model));
            candidates.push(root.join("resources").join("models").join(model));
        }
    }
    candidates
}

fn env_path(name: &str) -> Vec<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .into_iter()
        .collect()
}

fn search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(current_executable) = env::current_exe() {
        if let Some(parent) = current_executable.parent() {
            roots.push(parent.to_owned());
        }
    }
    if let Ok(current_directory) = env::current_dir() {
        let development_manifest = current_directory.join("src-tauri").join("Cargo.toml");
        if development_manifest.is_file() {
            roots.push(current_directory);
        }
    }
    roots
}

fn first_valid_file(candidates: Vec<PathBuf>, minimum_bytes: u64) -> Option<PathBuf> {
    candidates
        .into_iter()
        .find(|candidate| valid_file(candidate, minimum_bytes))
}

fn valid_files(candidates: Vec<PathBuf>, minimum_bytes: u64) -> Vec<PathBuf> {
    candidates
        .into_iter()
        .filter(|candidate| valid_file(candidate, minimum_bytes))
        .collect()
}

fn valid_file(candidate: &Path, minimum_bytes: u64) -> bool {
    candidate
        .metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.len() >= minimum_bytes)
}

fn model_for(engine: &str) -> &str {
    match engine {
        WHISPER_ENGINE => DEFAULT_MODEL_FILE,
        COHERE_ENGINE => "cohere-transcribe-arabic-q4_k-imatrix.gguf",
        QWEN_ENGINE => "qwen3-asr-0.6b-q4_k.gguf",
        OMNI_ENGINE => "omniasr-ctc-300m-v2-q4_k.gguf",
        _ => "unknown",
    }
}

fn model_name(model: &Path) -> &str {
    model
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(DEFAULT_MODEL_FILE)
}

pub fn translate(
    text: &str,
    source_language: &str,
    target_language: &str,
) -> Result<String, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("There is no transcript to translate.".into());
    }
    if text.chars().count() > 12_000 {
        return Err("The transcript is too long for a single local translation.".into());
    }
    let source = validate_translation_language(source_language)?;
    let target = validate_translation_language(target_language)?;
    if source == target {
        return Ok(text.to_owned());
    }
    let installation = require_crisp_installation(TRANSLATOR_MODEL_ID, "m2m100")?;
    let mut command = Command::new(&installation.executable);
    command
        .arg("--backend")
        .arg("m2m100")
        .arg("-m")
        .arg(&installation.model)
        .arg("--text")
        .arg(text)
        .arg("-sl")
        .arg(source)
        .arg("-tl")
        .arg(target)
        .arg("-np")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_child_console(&mut command);
    let child = command
        .spawn()
        .map_err(|error| format!("Could not start the local translator: {error}"))?;
    let output = wait_for_output(child, Duration::from_secs(300))?;
    if !output.status.success() {
        return Err(crisp_command_error("M2M100 translation", &output));
    }
    let translated = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if translated.is_empty() {
        Err("The local translator returned empty text.".into())
    } else {
        Ok(translated)
    }
}

fn validate_backend(value: &str) -> Result<String, String> {
    let backend = value.trim().to_ascii_lowercase();
    if backend == "auto" {
        return Ok(String::new());
    }
    if backend.is_empty()
        || backend.len() > 40
        || !backend
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_".contains(character))
    {
        return Err("The CrispASR backend name contains unsupported characters.".into());
    }
    Ok(backend)
}

fn validate_translation_language(value: &str) -> Result<&str, String> {
    const SUPPORTED: &[&str] = &[
        "af", "am", "ar", "ast", "az", "ba", "be", "bg", "bn", "br", "bs", "ca", "ceb", "cs", "cy",
        "da", "de", "el", "en", "es", "et", "fa", "ff", "fi", "fr", "fy", "ga", "gd", "gl", "gu",
        "ha", "he", "hi", "hr", "ht", "hu", "hy", "id", "ig", "ilo", "is", "it", "ja", "jv", "ka",
        "kk", "km", "kn", "ko", "lb", "lg", "ln", "lo", "lt", "lv", "mg", "mk", "ml", "mn", "mr",
        "ms", "my", "ne", "nl", "no", "ns", "oc", "or", "pa", "pl", "ps", "pt", "ro", "ru", "sd",
        "si", "sk", "sl", "so", "sq", "sr", "ss", "su", "sv", "sw", "ta", "th", "tl", "tn", "tr",
        "uk", "ur", "uz", "vi", "wo", "xh", "yi", "yo", "zh", "zu",
    ];
    SUPPORTED
        .contains(&value)
        .then_some(value)
        .ok_or_else(|| format!("M2M100 does not support the language code `{value}`."))
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else {
        format!("{} MB", bytes.div_ceil(1_000_000))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        encode_pcm16_mono_wav, endpoint_language, endpoint_text, glossary_prompt,
        inference_timeout, mixed_wav_chunks, validate_endpoint, whisper_command, whisper_language,
        EndpointConnection, SessionFiles, WhisperInstallation,
    };
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn whisper_mixed_route_uses_automatic_language_detection() {
        assert_eq!(whisper_language("mixed"), "auto");
        assert_eq!(whisper_language("ar"), "ar");
        assert_eq!(whisper_language("en"), "en");
        assert_eq!(whisper_language("unknown"), "auto");
    }

    #[test]
    fn shutting_down_an_idle_whisper_runtime_is_safe() {
        super::WhisperRuntime::default().shutdown();
    }

    #[test]
    fn glossary_prompt_preserves_arabic_and_latin_terms() {
        let prompt = glossary_prompt(
            &["TypeSpeak".into(), "بيروت".into(), "  NABILNET  ".into()],
            "mixed",
        );
        assert!(prompt.contains("today عندي meeting"));
        assert!(prompt.contains("TypeSpeak"));
        assert!(prompt.contains("بيروت"));
        assert!(prompt.contains("NABILNET"));
    }

    #[test]
    fn mixed_audio_is_split_only_at_a_sustained_silence() {
        let sample_rate = 16_000;
        let tone = vec![4_000_i16; sample_rate * 2];
        let silence = vec![0_i16; sample_rate / 2];
        let mut samples = tone.clone();
        samples.extend_from_slice(&silence);
        samples.extend_from_slice(&tone);
        let wav = encode_pcm16_mono_wav(&samples, sample_rate as u32);

        let chunks = mixed_wav_chunks(&wav).expect("a safe mixed-language boundary");

        assert_eq!(chunks.len(), 2);
        assert!(chunks.iter().all(|(audio, duration)| {
            audio.starts_with(b"RIFF") && *duration >= 1_900 && *duration <= 2_600
        }));
    }

    #[test]
    fn continuous_speech_is_not_split_mid_word() {
        let sample_rate = 16_000;
        let samples = vec![3_000_i16; sample_rate * 5];
        let wav = encode_pcm16_mono_wav(&samples, sample_rate as u32);

        assert!(mixed_wav_chunks(&wav).is_none());
    }

    #[test]
    fn whisper_mixed_regression_never_enables_translation() {
        let installation = WhisperInstallation {
            executable: PathBuf::from("whisper-cli"),
            servers: Vec::new(),
            model: PathBuf::from("model.bin"),
        };
        let files = SessionFiles::create().unwrap();
        let command = whisper_command(&installation, &files, "mixed", &[]);
        let arguments = command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert!(arguments
            .windows(2)
            .any(|pair| pair[0] == "-l" && pair[1] == "auto"));
        assert!(!arguments.iter().any(|argument| argument == "-tr"));
    }

    #[test]
    fn inference_timeout_scales_but_remains_bounded() {
        assert_eq!(inference_timeout(1_000), Duration::from_secs(124));
        assert_eq!(inference_timeout(999_000), Duration::from_secs(480));
    }

    #[test]
    fn mixed_endpoint_route_lets_the_model_detect_the_language() {
        assert_eq!(endpoint_language("mixed"), None);
        assert_eq!(endpoint_language("ar"), Some("ar"));
        assert_eq!(endpoint_language("en"), Some("en"));
    }

    #[test]
    fn endpoint_transport_matches_privacy_label() {
        assert!(validate_endpoint(
            "http://127.0.0.1:8000/v1/audio/transcriptions",
            &EndpointConnection::Local,
        )
        .is_ok());
        assert!(validate_endpoint(
            "https://api.example.com/v1/audio/transcriptions",
            &EndpointConnection::Api,
        )
        .is_ok());
        assert!(validate_endpoint(
            "https://api.example.com/v1/audio/transcriptions",
            &EndpointConnection::Local,
        )
        .is_err());
        assert!(validate_endpoint(
            "http://api.example.com/v1/audio/transcriptions",
            &EndpointConnection::Api,
        )
        .is_err());
    }

    #[test]
    fn endpoint_response_trims_transcript_text() {
        assert_eq!(
            endpoint_text(r#"{"text":" أهلا وسهلا "}"#).unwrap(),
            "أهلا وسهلا"
        );
    }

    #[test]
    fn endpoint_response_without_text_is_rejected() {
        assert!(endpoint_text(r#"{"result":"missing"}"#).is_err());
    }
}
