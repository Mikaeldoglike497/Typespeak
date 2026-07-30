const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke;
const listen = tauri?.event?.listen;
const emit = tauri?.event?.emit;
const emitTo = tauri?.event?.emitTo;

const elements = {
  sessionState: document.querySelector("#sessionState"),
  privacyLine: document.querySelector("#privacyLine"),
  languageControl: document.querySelector("#languageControl"),
  providerSelect: document.querySelector("#providerSelect"),
  outputSelect: document.querySelector("#outputSelect"),
  activeRouteLabel: document.querySelector("#activeRouteLabel"),
  activeOutputLabel: document.querySelector("#activeOutputLabel"),
  privacyBadge: document.querySelector("#privacyBadge"),
  privacyBadgeText: document.querySelector("#privacyBadgeText"),
  routeArabic: document.querySelector("#routeArabic"),
  routeMixed: document.querySelector("#routeMixed"),
  routeEnglish: document.querySelector("#routeEnglish"),
  outputArabic: document.querySelector("#outputArabic"),
  outputMixed: document.querySelector("#outputMixed"),
  outputEnglish: document.querySelector("#outputEnglish"),
  settingsOutputArabic: document.querySelector("#settingsOutputArabic"),
  settingsOutputMixed: document.querySelector("#settingsOutputMixed"),
  settingsOutputEnglish: document.querySelector("#settingsOutputEnglish"),
  speechLane: document.querySelector(".speech-lane"),
  laneStatus: document.querySelector("#laneStatus"),
  waveform: document.querySelector("#waveform"),
  recordButton: document.querySelector("#recordButton"),
  recordAction: document.querySelector("#recordAction"),
  recordHint: document.querySelector("#recordHint"),
  recordTimer: document.querySelector("#recordTimer"),
  shortcutDisplay: document.querySelector("#shortcutDisplay"),
  transcript: document.querySelector("#transcript"),
  emptyGuide: document.querySelector("#emptyGuide"),
  activeModelLabel: document.querySelector("#activeModelLabel"),
  latencyLabel: document.querySelector("#latencyLabel"),
  cleanupMode: document.querySelector("#cleanupMode"),
  clearTranscript: document.querySelector("#clearTranscript"),
  copyTranscript: document.querySelector("#copyTranscript"),
  insertTranscript: document.querySelector("#insertTranscript"),
  providerList: document.querySelector("#providerList"),
  refreshProviders: document.querySelector("#refreshProviders"),
  connectionHelp: document.querySelector("#connectionHelp"),
  glossaryTags: document.querySelector("#glossaryTags"),
  glossaryCount: document.querySelector("#glossaryCount"),
  glossaryInput: document.querySelector("#glossaryInput"),
  addGlossary: document.querySelector("#addGlossary"),
  metricRuns: document.querySelector("#metricRuns"),
  metricMinutes: document.querySelector("#metricMinutes"),
  metricProviders: document.querySelector("#metricProviders"),
  sessionPrivacyTitle: document.querySelector("#sessionPrivacyTitle"),
  sessionPrivacyNote: document.querySelector("#sessionPrivacyNote"),
  targetNote: document.querySelector("#targetNote"),
  settingsModal: document.querySelector("#settingsModal"),
  openSettings: document.querySelector("#openSettings"),
  closeSettings: document.querySelector("#closeSettings"),
  keyList: document.querySelector("#keyList"),
  translatorCard: document.querySelector(".translator-card"),
  translatorStatus: document.querySelector("#translatorStatus"),
  installTranslator: document.querySelector("#installTranslator"),
  settingsTranslatorStatus: document.querySelector("#settingsTranslatorStatus"),
  settingsInstallTranslator: document.querySelector("#settingsInstallTranslator"),
  shortcutPreview: document.querySelector("#shortcutPreview"),
  changeShortcut: document.querySelector("#changeShortcut"),
  modelForm: document.querySelector("#modelForm"),
  modelFormTitle: document.querySelector("#modelFormTitle"),
  modelEditId: document.querySelector("#modelEditId"),
  modelName: document.querySelector("#modelName"),
  modelConnection: document.querySelector("#modelConnection"),
  modelEndpoint: document.querySelector("#modelEndpoint"),
  modelEndpointLabel: document.querySelector("#modelEndpointLabel"),
  modelIdentifier: document.querySelector("#modelIdentifier"),
  modelIdentifierLabel: document.querySelector("#modelIdentifierLabel"),
  modelApiKey: document.querySelector("#modelApiKey"),
  modelApiKeyField: document.querySelector("#modelApiKeyField"),
  modelChecksum: document.querySelector("#modelChecksum"),
  modelChecksumField: document.querySelector("#modelChecksumField"),
  modelFormNote: document.querySelector("#modelFormNote"),
  saveModel: document.querySelector("#saveModel"),
  cancelModelEdit: document.querySelector("#cancelModelEdit"),
  toastStack: document.querySelector("#toastStack"),
  minimizeWindow: document.querySelector("#minimizeWindow"),
  maximizeWindow: document.querySelector("#maximizeWindow"),
  closeWindow: document.querySelector("#closeWindow"),
  recentList: document.querySelector("#recentList"),
  recentEmpty: document.querySelector("#recentEmpty"),
  recentCount: document.querySelector("#recentCount"),
  recentWords: document.querySelector("#recentWords"),
  recentMinutes: document.querySelector("#recentMinutes"),
  clearRecent: document.querySelector("#clearRecent"),
  recentPrivacy: document.querySelector("#recentPrivacy"),
  dictionaryForm: document.querySelector("#dictionaryForm"),
  dictionaryInput: document.querySelector("#dictionaryInput"),
  dictionarySearch: document.querySelector("#dictionarySearch"),
  dictionaryList: document.querySelector("#dictionaryList"),
  dictionaryEmpty: document.querySelector("#dictionaryEmpty"),
  dictionaryCount: document.querySelector("#dictionaryCount"),
  shortcutPagePreview: document.querySelector("#shortcutPagePreview"),
  changeShortcutPage: document.querySelector("#changeShortcutPage"),
  settingsShortcutSummary: document.querySelector("#settingsShortcutSummary"),
  startWithWindowsToggle: document.querySelector("#startWithWindowsToggle"),
  saveRecentToggle: document.querySelector("#saveRecentToggle"),
};

const APP_VIEWS = new Set(["dictate", "recent", "dictionary", "shortcuts", "settings"]);

const state = {
  recording: false,
  starting: false,
  recordRequested: false,
  processing: false,
  language: "mixed",
  cleanupMode: "verbatim",
  engines: [],
  models: [],
  routes: {
    ar: "whisper-local",
    en: "whisper-local",
    mixed: "whisper-local",
  },
  outputRoutes: {
    ar: "original",
    en: "original",
    mixed: "original",
  },
  translator: null,
  downloads: new Map(),
  modelSecrets: new Map(),
  transcriptions: [],
  glossary: ["TypeSpeak", "NABILNET", "Lebanese"],
  audioContext: null,
  stream: null,
  source: null,
  processor: null,
  analyser: null,
  samples: [],
  sampleRate: 48_000,
  startedAt: 0,
  timerId: null,
  animationId: null,
  targetCaptured: false,
  autoInsert: false,
  pushToTalkShortcut: "Control+Alt+Space",
  capturingShortcut: false,
  runs: 0,
  totalDurationMs: 0,
  enginesUsed: new Set(),
  recent: [],
  saveRecent: true,
  startupEnabled: true,
  lastOverlayLevelAt: 0,
};

const MODEL_STORAGE_KEY = "typespeak.models.v2";
const ROUTE_STORAGE_KEY = "typespeak.routes.v1";
const OUTPUT_STORAGE_KEY = "typespeak.outputs.v1";
const SHORTCUT_STORAGE_KEY = "typespeak.shortcut.v1";
const GLOSSARY_STORAGE_KEY = "typespeak.glossary.v1";
const RECENT_STORAGE_KEY = "typespeak.recent.v1";
const SAVE_RECENT_STORAGE_KEY = "typespeak.saveRecent.v1";
const STARTUP_PREFERENCE_STORAGE_KEY = "typespeak.startWithWindows.v1";
const DEFAULT_PUSH_TO_TALK_SHORTCUT = "Control+Alt+Space";
const SHORTCUT_CODE_PATTERN =
  /^(?:Key[A-Z]|Digit\d|F(?:[1-9]|1\d|2[0-4])|Space|Enter|Tab|Backspace|Delete|Insert|Home|End|PageUp|PageDown|Arrow(?:Up|Down|Left|Right)|CapsLock|PrintScreen|ScrollLock|Pause|NumLock|AudioVolume(?:Down|Up|Mute)|Media(?:Play|Pause|PlayPause|Stop|TrackNext|TrackPrevious)|Backquote|Backslash|BracketLeft|BracketRight|Comma|Equal|Minus|Period|Quote|Semicolon|Slash|Numpad(?:\d|Add|Decimal|Divide|Enter|Equal|Multiply|Subtract))$/;
const DEFAULT_ROUTES = {
  ar: "whisper-local",
  en: "whisper-local",
  mixed: "whisper-local",
};
const DEFAULT_OUTPUT_ROUTES = {
  ar: "original",
  en: "original",
  mixed: "original",
};
const routeLabels = {
  ar: "Arabic",
  en: "English",
  mixed: "Mixed",
};
const BUILTIN_MODELS = [
  {
    id: "whisper-local",
    name: "Whisper large-v3-turbo",
    model: "ggml-large-v3-turbo-q5_0.bin",
    connection: "native",
    backendEngine: "whisper",
    configured: false,
    managed: true,
    downloadBytes: 574_041_195,
    builtIn: true,
    note: "Native whisper.cpp runtime. Multilingual and fully offline.",
    setupHint: "Download 574 MB",
  },
  {
    id: "cohere-local",
    name: "Cohere Arabic · local",
    model: "cohere-transcribe-arabic-q4_k-imatrix.gguf",
    connection: "native",
    backendEngine: "cohere",
    configured: false,
    managed: true,
    downloadBytes: 1_510_365_312,
    builtIn: true,
    note: "Arabic-first native GGUF model. No Python or local server required.",
    setupHint: "Download 1.51 GB",
  },
  {
    id: "cohere-api",
    name: "Cohere Arabic · API",
    model: "cohere-transcribe-arabic-07-2026",
    connection: "api",
    endpoint: "https://api.cohere.com/v2/audio/transcriptions",
    configured: false,
    builtIn: true,
    note: "Hosted Cohere transcription. Audio is uploaded to Cohere.",
    setupHint: "Add a Cohere API key",
  },
  {
    id: "qwen-local",
    name: "Qwen3-ASR 0.6B · local",
    model: "qwen3-asr-0.6b-q4_k.gguf",
    connection: "native",
    backendEngine: "qwen3",
    configured: false,
    managed: true,
    downloadBytes: 631_026_336,
    builtIn: true,
    note: "Light multilingual native model with automatic language detection.",
    setupHint: "Download 631 MB",
  },
  {
    id: "omni-local",
    name: "OmniASR CTC 300M · local",
    model: "omniasr-ctc-300m-v2-q4_k.gguf",
    connection: "native",
    backendEngine: "omniasr",
    configured: false,
    managed: true,
    downloadBytes: 203_542_816,
    builtIn: true,
    note: "Compact multilingual native CTC model for lower-memory computers.",
    setupHint: "Download 204 MB",
  },
];

const engineNames = {
  whisper: "Whisper large-v3-turbo",
  cohere: "Cohere Transcribe Arabic",
  qwen3: "Qwen3-ASR 0.6B",
  omniasr: "OmniASR CTC 300M",
  mock: "Offline demo",
};

const TRANSLATOR_ID = "translator-m2m100";
const TRANSLATOR_BYTES = 526_331_008;
const TRANSLATION_LANGUAGE_CODES = (
  "af am ar ast az ba be bg bn br bs ca ceb cs cy da de el en es et fa ff fi fr fy ga gd gl gu " +
  "ha he hi hr ht hu hy id ig ilo is it ja jv ka kk km kn ko lb lg ln lo lt lv mg mk ml mn mr " +
  "ms my ne nl no ns oc or pa pl ps pt ro ru sd si sk sl so sq sr ss su sv sw ta th tl tn tr uk " +
  "ur uz vi wo xh yi yo zh zu"
).split(" ");

function loadSavedRoutes() {
  try {
    return {
      ...DEFAULT_ROUTES,
      ...JSON.parse(localStorage.getItem(ROUTE_STORAGE_KEY) || "{}"),
    };
  } catch {
    return { ...DEFAULT_ROUTES };
  }
}

function loadSavedOutputRoutes() {
  try {
    const savedRoutes = {
      ...DEFAULT_OUTPUT_ROUTES,
      ...JSON.parse(localStorage.getItem(OUTPUT_STORAGE_KEY) || "{}"),
    };
    for (const route of Object.keys(DEFAULT_OUTPUT_ROUTES)) {
      if (
        savedRoutes[route] === "mixed"
        || (route !== "mixed" && savedRoutes[route] === route)
      ) {
        savedRoutes[route] = "original";
      }
    }
    return savedRoutes;
  } catch {
    return { ...DEFAULT_OUTPUT_ROUTES };
  }
}

function loadSavedModels() {
  try {
    const models = JSON.parse(localStorage.getItem(MODEL_STORAGE_KEY) || "[]");
    return Array.isArray(models) ? models : [];
  } catch {
    return [];
  }
}

function loadSavedShortcut() {
  return localStorage.getItem(SHORTCUT_STORAGE_KEY) || DEFAULT_PUSH_TO_TALK_SHORTCUT;
}

function loadSavedGlossary() {
  try {
    const glossary = JSON.parse(localStorage.getItem(GLOSSARY_STORAGE_KEY) || "[]");
    return Array.isArray(glossary) && glossary.length
      ? glossary.filter((term) => typeof term === "string" && term.trim()).slice(0, 250)
      : ["TypeSpeak", "NABILNET", "Lebanese"];
  } catch {
    return ["TypeSpeak", "NABILNET", "Lebanese"];
  }
}

function loadRecentHistory() {
  try {
    const recent = JSON.parse(localStorage.getItem(RECENT_STORAGE_KEY) || "[]");
    return Array.isArray(recent)
      ? recent.filter((entry) => entry && typeof entry.text === "string").slice(0, 100)
      : [];
  } catch {
    return [];
  }
}

function loadSaveRecentPreference() {
  return localStorage.getItem(SAVE_RECENT_STORAGE_KEY) !== "false";
}

function persistModels() {
  const safeModels = state.models
    .filter((model) => model.connection !== "native")
    .map((model) => ({
      ...model,
      configured: model.connection === "local" && model.configured,
    }));
  localStorage.setItem(MODEL_STORAGE_KEY, JSON.stringify(safeModels));
}

function persistRoutes() {
  localStorage.setItem(ROUTE_STORAGE_KEY, JSON.stringify(state.routes));
}

function persistOutputRoutes() {
  localStorage.setItem(OUTPUT_STORAGE_KEY, JSON.stringify(state.outputRoutes));
}

function persistGlossary() {
  localStorage.setItem(GLOSSARY_STORAGE_KEY, JSON.stringify(state.glossary));
}

function persistRecentHistory() {
  if (state.saveRecent) {
    localStorage.setItem(RECENT_STORAGE_KEY, JSON.stringify(state.recent));
  } else {
    localStorage.removeItem(RECENT_STORAGE_KEY);
  }
}

function createWaveform() {
  const fragment = document.createDocumentFragment();
  for (let index = 0; index < 58; index += 1) {
    const bar = document.createElement("span");
    bar.className = "wave-bar";
    const distance = Math.abs(index - 28.5) / 28.5;
    bar.style.height = `${Math.max(6, 18 - distance * 12)}px`;
    bar.style.animationDelay = `${index * 22}ms`;
    fragment.appendChild(bar);
  }
  elements.waveform.appendChild(fragment);
}

function setStatus(kind, label) {
  elements.sessionState.classList.toggle("is-recording", kind === "recording");
  elements.sessionState.classList.toggle("is-processing", kind === "processing");
  elements.sessionState.classList.toggle("is-error", kind === "error");
  elements.sessionState.classList.toggle("is-preview", kind === "preview");
  elements.sessionState.querySelector("span:last-child").textContent = label;
}

function setTranscript(text, transcription = null) {
  elements.transcript.value = text;
  const hasArabic = /[\u0600-\u06ff]/.test(text);
  elements.transcript.classList.toggle("has-arabic", hasArabic);
  elements.transcript.dir = hasArabic ? "auto" : "ltr";
  elements.emptyGuide.classList.toggle("is-hidden", Boolean(text.trim()));
  if (transcription) {
    elements.activeModelLabel.textContent =
      engineNames[transcription.engine] || transcription.engine;
    elements.latencyLabel.textContent = transcription.demo
      ? "demo result"
      : `${transcription.elapsedMs} ms final`;
  }
}

function toast(title, message, error = false) {
  const toastElement = document.createElement("div");
  toastElement.className = `toast${error ? " is-error" : ""}`;
  toastElement.innerHTML = `<div></div><div><strong></strong><p></p></div>`;
  toastElement.querySelector("strong").textContent = title;
  toastElement.querySelector("p").textContent = message;
  elements.toastStack.appendChild(toastElement);
  window.setTimeout(() => toastElement.remove(), 4200);
}

function formatTimer(milliseconds) {
  const totalTenths = Math.floor(milliseconds / 100);
  const minutes = Math.floor(totalTenths / 600);
  const seconds = Math.floor((totalTenths % 600) / 10);
  const tenths = totalTenths % 10;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}.${tenths}`;
}

function updateRecordingUi() {
  elements.speechLane.classList.toggle("is-recording", state.recording);
  elements.speechLane.classList.toggle("is-processing", state.processing);
  elements.recordAction.textContent = state.recording
    ? "Stop and transcribe"
    : state.processing
      ? "Transcribing locally…"
      : "Start a test recording";
  elements.recordHint.textContent = state.recording
    ? "release the shortcut to finish"
    : `or hold ${shortcutLabel()} anywhere`;
  elements.recordButton.disabled = state.processing;
  elements.providerSelect.disabled = state.recording || state.processing;
  elements.outputSelect.disabled = state.recording || state.processing;
  routeSelectElements().forEach((select) => {
    select.disabled = state.recording || state.processing;
  });
  outputSelectElements().forEach((select) => {
    select.disabled = state.recording || state.processing;
  });
}

function renderPushToTalkShortcut() {
  renderShortcutKeycaps(elements.shortcutDisplay, state.pushToTalkShortcut);
  elements.shortcutDisplay.setAttribute(
    "aria-label",
    `Push-to-talk shortcut ${shortcutLabel()}`,
  );
  renderShortcutCaptureState();
  selectActiveShortcutOption();
}

function renderShortcutCaptureState() {
  const previews = [elements.shortcutPreview, elements.shortcutPagePreview];
  previews.forEach((preview) => {
    preview.classList.toggle("is-capturing", state.capturingShortcut);
  });
  elements.changeShortcut.textContent = state.capturingShortcut ? "Cancel" : "Change";
  elements.changeShortcutPage.textContent = state.capturingShortcut
    ? "Cancel"
    : "Change shortcut";
  if (state.capturingShortcut) {
    previews.forEach((preview) => {
      preview.textContent = "Press any Windows key…";
    });
  } else {
    previews.forEach((preview) => {
      renderShortcutKeycaps(preview, state.pushToTalkShortcut);
    });
  }
  elements.settingsShortcutSummary.textContent =
    `Quick tap keeps ${shortcutLabel()} normal; hold it for 0.3 seconds to dictate.`;
}

function selectActiveShortcutOption() {
  document.querySelectorAll("[data-shortcut-option]").forEach((button) => {
    button.classList.toggle(
      "is-selected",
      button.dataset.shortcutOption === state.pushToTalkShortcut,
    );
  });
}

function renderShortcutKeycaps(container, shortcut) {
  container.replaceChildren();
  const parts = shortcut.split("+");
  parts.forEach((part, index) => {
    const key = document.createElement("kbd");
    key.textContent = shortcutPartLabel(part);
    container.appendChild(key);
    if (index < parts.length - 1) {
      const plus = document.createElement("span");
      plus.textContent = "+";
      container.appendChild(plus);
    }
  });
}

function shortcutPartLabel(part) {
  const labels = {
    Control: "Ctrl",
    Super: "Win",
    Space: "Space",
    Insert: "Ins",
    PrintScreen: "Prt Scr",
    PageUp: "PgUp",
    PageDown: "PgDn",
    ScrollLock: "ScrLk",
    AudioVolumeUp: "Vol +",
    AudioVolumeDown: "Vol −",
    AudioVolumeMute: "Mute",
    MediaPlayPause: "Play / Pause",
    MediaTrackNext: "Next",
    MediaTrackPrevious: "Previous",
  };
  if (labels[part]) return labels[part];
  if (part.startsWith("Key")) return part.slice(3);
  if (part.startsWith("Digit")) return part.slice(5);
  return part;
}

function shortcutLabel(shortcut = state.pushToTalkShortcut) {
  return shortcut.split("+").map(shortcutPartLabel).join(" + ");
}

function updateMetrics() {
  elements.metricRuns.textContent = String(state.runs);
  elements.metricMinutes.textContent = (state.totalDurationMs / 60_000).toFixed(1);
  elements.metricProviders.textContent = String(state.enginesUsed.size);
}

function renderGlossary() {
  elements.glossaryTags.replaceChildren();
  state.glossary.forEach((term) => {
    const tag = document.createElement("div");
    tag.className = "glossary-tag";
    const label = document.createElement("span");
    label.textContent = term;
    const remove = document.createElement("button");
    remove.type = "button";
    remove.setAttribute("aria-label", `Remove ${term}`);
    remove.textContent = "×";
    remove.addEventListener("click", () => {
      removeGlossaryTerm(term);
    });
    tag.append(label, remove);
    elements.glossaryTags.appendChild(tag);
  });
  elements.glossaryCount.textContent = String(state.glossary.length);
  renderDictionary();
}

function addGlossaryTerm() {
  addGlossaryValue(elements.glossaryInput.value);
  elements.glossaryInput.value = "";
}

function addGlossaryValue(value) {
  const term = value.trim();
  if (!term || state.glossary.includes(term)) {
    return;
  }
  state.glossary.push(term);
  persistGlossary();
  renderGlossary();
}

function removeGlossaryTerm(term) {
  state.glossary = state.glossary.filter((savedTerm) => savedTerm !== term);
  persistGlossary();
  renderGlossary();
}

function renderDictionary() {
  const query = elements.dictionarySearch.value.trim().toLocaleLowerCase();
  const terms = state.glossary.filter((term) => term.toLocaleLowerCase().includes(query));
  const fragment = document.createDocumentFragment();
  for (const term of terms) {
    fragment.appendChild(createDictionaryTermRow(term));
  }
  elements.dictionaryList.replaceChildren(fragment);
  elements.dictionaryCount.textContent = String(state.glossary.length);
  elements.dictionaryEmpty.classList.toggle("is-hidden", terms.length > 0);
}

function createDictionaryTermRow(term) {
  const row = document.createElement("div");
  row.className = "dictionary-term";
  const label = document.createElement("span");
  label.textContent = term;
  label.dir = "auto";
  const remove = document.createElement("button");
  remove.type = "button";
  remove.textContent = "×";
  remove.setAttribute("aria-label", `Remove ${term}`);
  remove.addEventListener("click", () => removeGlossaryTerm(term));
  row.append(label, remove);
  return row;
}

function addRecentTranscript(text, model, transcription, durationMs) {
  if (!state.saveRecent || !text.trim()) return;
  state.recent.unshift({
    id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
    createdAt: new Date().toISOString(),
    text: text.trim(),
    language: state.language,
    model: model.name,
    elapsedMs: Number(transcription.elapsedMs) || 0,
    audioDurationMs: Number(durationMs) || 0,
  });
  state.recent = state.recent.slice(0, 100);
  persistRecentHistory();
  renderRecentHistory();
}

function renderRecentHistory() {
  const fragment = document.createDocumentFragment();
  for (const entry of state.recent) {
    fragment.appendChild(createRecentRow(entry));
  }
  elements.recentList.replaceChildren(fragment);
  elements.recentList.classList.toggle("is-hidden", state.recent.length === 0);
  elements.recentEmpty.classList.toggle("is-hidden", state.recent.length > 0);
  renderRecentSummary();
}

function createRecentRow(entry) {
  const row = document.createElement("article");
  row.className = "recent-row";
  row.append(createRecentTimestamp(entry.createdAt), createRecentCopy(entry), createRecentCopyButton(entry));
  return row;
}

function createRecentTimestamp(createdAt) {
  const timestamp = new Date(createdAt);
  const isValid = !Number.isNaN(timestamp.getTime());
  const time = document.createElement("div");
  time.className = "recent-time";
  const clock = document.createElement("b");
  clock.textContent = isValid
    ? timestamp.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
    : "—";
  const day = document.createElement("span");
  day.textContent = isValid
    ? timestamp.toLocaleDateString([], { month: "short", day: "numeric" })
    : "";
  time.append(clock, day);
  return time;
}

function createRecentCopy(entry) {
  const copy = document.createElement("div");
  copy.className = "recent-copy";
  const text = document.createElement("p");
  text.textContent = entry.text;
  text.dir = "auto";
  const metadata = document.createElement("small");
  metadata.textContent =
    `${routeLabels[entry.language] || entry.language} · ${entry.model} · ${entry.elapsedMs} ms`;
  copy.append(text, metadata);
  return copy;
}

function createRecentCopyButton(entry) {
  const copyButton = document.createElement("button");
  copyButton.type = "button";
  copyButton.textContent = "⧉";
  copyButton.setAttribute("aria-label", "Copy this transcript");
  copyButton.addEventListener("click", async () => {
    await navigator.clipboard.writeText(entry.text);
    toast("Copied", "Recent transcript copied to the clipboard.");
  });
  return copyButton;
}

function renderRecentSummary() {
  elements.recentCount.textContent = String(state.recent.length);
  const wordCount = state.recent.reduce(
    (count, entry) => count + entry.text.split(/\s+/).filter(Boolean).length,
    0,
  );
  const durationMs = state.recent.reduce((total, entry) => total + entry.audioDurationMs, 0);
  elements.recentWords.textContent = String(wordCount);
  const minutes = document.createTextNode((durationMs / 60_000).toFixed(1));
  const unit = document.createElement("small");
  unit.textContent = " min";
  elements.recentMinutes.replaceChildren(minutes, unit);
  elements.recentPrivacy.textContent = state.saveRecent
    ? "Saved locally · never uploaded"
    : "History is disabled";
}

async function callNative(command, payload = {}) {
  if (invoke) {
    return invoke(command, payload);
  }
  return callBrowserFallback(command, payload);
}

async function callBrowserFallback(command, payload) {
  if (command === "engine_status") {
    return browserEngineStatuses();
  }
  if (command === "transcribe_audio") {
    return browserDemoTranscription(payload.request);
  }
  if (command === "transcribe_model_endpoint") {
    return {
      engine: payload.request.connectionId,
      model: payload.request.model,
      text: "",
      elapsedMs: 0,
      audioDurationMs: payload.request.durationMs,
      ok: false,
      demo: true,
      error: "Run the Windows app to connect to local or cloud model endpoints.",
    };
  }
  if (command === "normalize_transcript") {
    return payload.text.replace(/\s+/g, " ").trim();
  }
  if (command === "insert_text") {
    await navigator.clipboard.writeText(payload.text);
    return {
      inserted: false,
      clipboardRestored: false,
      message: "Browser preview copied the transcript. Run the Windows app for direct insertion.",
    };
  }
  if (command === "set_push_to_talk_shortcut") {
    return payload.shortcut;
  }
  if (command === "startup_enabled" || command === "set_startup_enabled") {
    return true;
  }
  return null;
}

function browserEngineStatuses() {
  return [
    {
      id: "whisper",
      name: "Whisper local",
      model: "ggml-large-v3-turbo-q5_0.bin",
      configured: false,
      setupHint: "Run the Windows setup script",
      note: "Browser preview cannot run the native local model.",
    },
    {
      id: "mock",
      name: "Offline demo",
      model: "typespeak-demo-1",
      configured: true,
      setupHint: "Built in",
      note: "Browser demo mode.",
    },
  ];
}

async function browserDemoTranscription(request) {
  await new Promise((resolve) => window.setTimeout(resolve, 720));
  const demoText = browserDemoText(request.language);
  const isDemo = request.engine === "mock";
  return {
    engine: request.engine,
    model: "typespeak-browser-demo",
    text: isDemo ? demoText : "",
    elapsedMs: 720,
    audioDurationMs: request.durationMs,
    ok: isDemo,
    demo: true,
    error: isDemo ? null : "Run the Windows app to use the installed local model.",
  };
}

function browserDemoText(language) {
  if (language === "en") {
    return "TypeSpeak runs speech recognition locally, so your audio stays on this computer.";
  }
  if (language === "ar") {
    return "خلّينا نبلّش بنسخة الـWindows المحلية، وبعدها منجرّبها مع لبنانيين.";
  }
  return "خلّينا نجرّب الـLebanese dictation محلياً على Windows, وبعدها منعمل final review.";
}

async function loadEngines() {
  try {
    state.engines = await callNative("engine_status");
    buildModelRegistry();
    await refreshCustomManagedStatuses();
    await loadTranslatorStatus();
    renderEngines();
  } catch (error) {
    toast("Could not load the local engine", String(error), true);
  }
}

function buildModelRegistry() {
  const savedModels = loadSavedModels();
  const savedById = new Map(savedModels.map((model) => [model.id, model]));
  state.models = mergedBuiltInModels(savedById);
  applyNativeEngineStatuses();
  appendCustomModels(savedModels);
  appendBrowserDemoModel();
  applyBrowserPreviewAvailability();
  routeBrowserPreviewToDemo();
  repairRoutes();
}

function mergedBuiltInModels(savedById) {
  return BUILTIN_MODELS.map((model) => ({
    ...model,
    ...(savedById.get(model.id) || {}),
  }));
}

function applyNativeEngineStatuses() {
  const nativeByEngine = new Map(state.engines.map((engine) => [engine.id, engine]));
  state.models = state.models.map((model) => {
    if (model.connection !== "native") return model;
    const engine = nativeByEngine.get(model.backendEngine);
    return engine
      ? {
          ...model,
          configured: engine.configured,
          model: engine.model,
          setupHint: engine.setupHint,
          note: engine.note,
          managed: Boolean(engine.managed),
          downloadBytes: engine.downloadBytes || model.downloadBytes,
        }
      : model;
  });
}

function appendCustomModels(savedModels) {
  for (const savedModel of savedModels) {
    if (!state.models.some((model) => model.id === savedModel.id)) {
      state.models.push(savedModel);
    }
  }
}

async function refreshCustomManagedStatuses() {
  if (!invoke) return;
  const managedModels = state.models.filter(
    (model) => model.connection === "managed" && model.localFileName,
  );
  await Promise.all(managedModels.map(async (model) => {
    try {
      const status = await callNative("custom_model_status", {
        fileName: model.localFileName,
      });
      model.configured = Boolean(status.installed);
    } catch {
      model.configured = false;
    }
  }));
}

async function loadTranslatorStatus() {
  if (!invoke) {
    state.translator = {
      id: TRANSLATOR_ID,
      installed: false,
      expectedBytes: TRANSLATOR_BYTES,
    };
    return;
  }
  state.translator = await callNative("managed_model_status", {
    modelId: TRANSLATOR_ID,
  });
}

function appendBrowserDemoModel() {
  for (const engine of state.engines) {
    if (engine.id !== "mock") continue;
    state.models.push({
      id: "mock",
      name: engine.name,
      model: engine.model,
      connection: "native",
      backendEngine: "mock",
      configured: engine.configured,
      builtIn: true,
      note: engine.note,
      setupHint: engine.setupHint,
    });
  }
}

function routeBrowserPreviewToDemo() {
  if (invoke || !state.models.some((model) => model.id === "mock")) return;
  for (const route of Object.keys(DEFAULT_ROUTES)) {
    state.routes[route] = "mock";
  }
}

function applyBrowserPreviewAvailability() {
  if (invoke) return;
  state.models = state.models.map((model) => model.id === "mock"
    ? model
    : {
        ...model,
        configured: false,
        note: "Desktop app required. Browser preview cannot run this model.",
      });
}

function repairRoutes() {
  const fallback = state.models.find((model) => model.configured)?.id
    || state.models[0]?.id
    || "whisper-local";
  for (const route of Object.keys(DEFAULT_ROUTES)) {
    if (!state.models.some((model) => model.id === state.routes[route])) {
      state.routes[route] = fallback;
    }
  }
  persistRoutes();
}

function repairOutputRoutes() {
  const supported = new Set(["original", ...TRANSLATION_LANGUAGE_CODES]);
  for (const route of Object.keys(DEFAULT_OUTPUT_ROUTES)) {
    if (!supported.has(state.outputRoutes[route])) {
      state.outputRoutes[route] = DEFAULT_OUTPUT_ROUTES[route];
    }
  }
  persistOutputRoutes();
}

function renderEngines() {
  elements.providerList.replaceChildren();
  elements.keyList.replaceChildren();

  for (const model of state.models) {
    elements.providerList.appendChild(engineStatusRow(model));
    elements.keyList.appendChild(engineSetupRow(model));
  }
  renderRouteSelects();
  renderOutputSelects();
  renderTranslatorStatus();
  updateActiveRouteUi();
}

function engineStatusRow(model) {
  const engineRow = document.createElement("div");
  const progress = state.downloads.get(model.id);
  const classes = [
    "provider-row",
    model.configured ? "is-ready" : "",
    progress ? "is-downloading" : "",
    model.connection === "api" ? "is-cloud" : "",
  ].filter(Boolean);
  engineRow.className = classes.join(" ");
  engineRow.dataset.modelId = model.id;
  engineRow.innerHTML = `
    <span class="provider-indicator"></span>
    <div class="provider-info"><strong></strong><span></span></div>
    <span class="provider-state"></span>
  `;
  engineRow.querySelector("strong").textContent = model.name;
  engineRow.querySelector(".provider-info span").textContent = model.model;
  engineRow.querySelector(".provider-state").textContent = progress
    ? `${progress.percent ?? 0}%`
    : modelStatusLabel(model);
  engineRow.title = model.note;
  return engineRow;
}

function engineOption(model) {
  const option = document.createElement("option");
  option.value = model.id;
  const unavailable = model.managed ? " · download needed" : " · setup needed";
  option.textContent = `${model.name}${model.configured ? "" : unavailable}`;
  option.disabled = !invoke && model.id !== "mock";
  return option;
}

function engineSetupRow(model) {
  const setupRow = document.createElement("div");
  setupRow.className = "key-row";
  setupRow.dataset.modelId = model.id;
  setupRow.innerHTML =
    `<div><strong></strong><code></code></div><span class="key-row-actions"></span>`;
  setupRow.querySelector("strong").textContent = model.name;
  setupRow.querySelector("code").textContent = model.setupHint;
  const actions = setupRow.querySelector(".key-row-actions");
  actions.appendChild(modelStateBadge(model));
  appendModelActionButtons(actions, model);
  return setupRow;
}

function modelStateBadge(model) {
  const stateBadge = document.createElement("span");
  const progress = state.downloads.get(model.id);
  stateBadge.className = `key-state${model.configured ? " is-ready" : ""}`;
  stateBadge.textContent = progress
    ? `${progress.percent ?? 0}%`
    : model.configured
    ? ["native", "managed"].includes(model.connection) ? "installed" : "saved"
    : model.managed || model.connection === "managed" ? "download" : "setup";
  return stateBadge;
}

function modelStatusLabel(model) {
  if (!model.configured) return model.managed || model.connection === "managed"
    ? "download"
    : "setup";
  if (["native", "managed"].includes(model.connection)) return "local";
  return model.connection === "api" ? "API" : "configured";
}

function appendModelActionButtons(actions, model) {
  const downloading = state.downloads.has(model.id);
  if ((model.managed || model.connection === "managed") && !model.configured && !downloading) {
    actions.appendChild(modelActionButton("Download", () => installSpeechModel(model.id)));
  }
  if (!model.builtIn && model.connection !== "native") {
    actions.appendChild(modelActionButton("Configure", () => openModelEditor(model.id)));
  } else if (!["native", "managed"].includes(model.connection)) {
    actions.appendChild(modelActionButton("Configure", () => openModelEditor(model.id)));
  }
  if (!model.builtIn) {
    actions.appendChild(modelActionButton("Remove", () => removeModel(model.id)));
  }
}

function modelActionButton(label, action) {
  const button = document.createElement("button");
  button.type = "button";
  button.textContent = label;
  button.addEventListener("click", action);
  return button;
}

async function installSpeechModel(modelId) {
  const model = state.models.find((candidate) => candidate.id === modelId);
  if (!model || model.configured || state.downloads.has(model.id)) return Boolean(model?.configured);
  if (!invoke) {
    toast("Desktop app required", "Managed models can only be installed from TypeSpeak.", true);
    return false;
  }
  return downloadSpeechModel(model);
}

async function downloadSpeechModel(model) {
  state.downloads.set(model.id, {
    id: model.id,
    stage: "starting",
    downloadedBytes: 0,
    totalBytes: model.downloadBytes || null,
    percent: 0,
  });
  renderEngines();
  try {
    if (model.connection === "managed") {
      await callNative("install_custom_model", {
        request: {
          modelId: model.id,
          downloadUrl: model.downloadUrl,
          fileName: model.localFileName,
          expectedSha256: model.expectedSha256 || null,
        },
      });
    } else {
      await callNative("install_managed_model", { modelId: model.id });
    }
    toast("Model installed", `${model.name} is ready for local transcription.`);
    return true;
  } catch (error) {
    toast("Model download failed", error?.message || String(error), true);
    return false;
  } finally {
    state.downloads.delete(model.id);
    await loadEngines();
  }
}

async function installRequestedDefaultModel() {
  if (!invoke) return;
  const requested = await callNative("default_model_download_requested");
  const whisper = state.models.find((model) => model.id === "whisper-local");
  if (!requested || !whisper || whisper.configured || state.downloads.has(whisper.id)) return;
  toast("Downloading Whisper", "The installer requested the default local model download.");
  await downloadSpeechModel(whisper);
}

async function installTranslator() {
  if (state.translator?.installed) return true;
  if (!invoke) {
    toast("Desktop app required", "Local translation installs from TypeSpeak, not Chrome.", true);
    return false;
  }
  if (state.downloads.has(TRANSLATOR_ID)) return false;
  state.downloads.set(TRANSLATOR_ID, {
    id: TRANSLATOR_ID,
    stage: "starting",
    downloadedBytes: 0,
    totalBytes: TRANSLATOR_BYTES,
    percent: 0,
  });
  renderTranslatorStatus();
  try {
    state.translator = await callNative("install_managed_model", {
      modelId: TRANSLATOR_ID,
    });
    toast("Translator installed", "100-language local translation is ready.");
    return true;
  } catch (error) {
    toast("Translator download failed", error?.message || String(error), true);
    return false;
  } finally {
    state.downloads.delete(TRANSLATOR_ID);
    await loadTranslatorStatus();
    renderTranslatorStatus();
  }
}

async function bindModelDownloadProgress() {
  if (!listen) return;
  await listen("typespeak://model-download", (event) => {
    const progress = event.payload;
    state.downloads.set(progress.id, progress);
    updateDownloadProgress(progress);
  });
}

function updateDownloadProgress(progress) {
  if (progress.id === TRANSLATOR_ID) {
    renderTranslatorStatus();
    return;
  }
  const row = elements.providerList.querySelector(`[data-model-id="${progress.id}"]`);
  if (row) {
    row.classList.add("is-downloading");
    row.querySelector(".provider-state").textContent = progress.stage === "verifying"
      ? "verify"
      : `${progress.percent ?? 0}%`;
    row.title = downloadProgressLabel(progress);
  }
  const settingsRow = [...elements.keyList.querySelectorAll(".key-row")]
    .find((candidate) => candidate.dataset.modelId === progress.id);
  if (settingsRow) {
    const badge = settingsRow.querySelector(".key-state");
    badge.textContent = progress.stage === "verifying" ? "verify" : `${progress.percent ?? 0}%`;
  }
}

function downloadProgressLabel(progress) {
  if (progress.stage === "verifying") return "Verifying SHA-256 integrity…";
  if (progress.stage === "installed") return "Installed";
  const downloaded = formatBytes(progress.downloadedBytes || 0);
  const total = progress.totalBytes ? ` of ${formatBytes(progress.totalBytes)}` : "";
  return `Downloading ${downloaded}${total}`;
}

function formatBytes(bytes) {
  if (bytes >= 1_000_000_000) return `${(bytes / 1_000_000_000).toFixed(2)} GB`;
  return `${Math.ceil(bytes / 1_000_000)} MB`;
}

function routeSelectElements() {
  return [elements.routeArabic, elements.routeMixed, elements.routeEnglish];
}

function outputSelectElements() {
  return [
    elements.outputArabic,
    elements.outputMixed,
    elements.outputEnglish,
    elements.settingsOutputArabic,
    elements.settingsOutputMixed,
    elements.settingsOutputEnglish,
  ];
}

function renderRouteSelects() {
  routeSelectElements().forEach((select) => {
    const route = select.dataset.route;
    select.replaceChildren();
    state.models.forEach((model) => select.appendChild(engineOption(model)));
    select.value = state.routes[route];
  });
  elements.providerSelect.replaceChildren();
  state.models.forEach((model) => elements.providerSelect.appendChild(engineOption(model)));
}

function renderOutputSelects() {
  outputSelectElements().forEach((select) => {
    const route = select.dataset.outputRoute;
    populateOutputSelect(select, route);
    select.value = state.outputRoutes[route] || DEFAULT_OUTPUT_ROUTES[route];
  });
  elements.outputSelect.dataset.outputRoute = state.language;
  populateOutputSelect(elements.outputSelect, state.language);
  elements.outputSelect.value =
    state.outputRoutes[state.language] || DEFAULT_OUTPUT_ROUTES[state.language];
}

function populateOutputSelect(select, route) {
  select.replaceChildren();
  const original = document.createElement("option");
  original.value = "original";
  original.textContent = {
    ar: "Keep Arabic · no translation",
    en: "Keep English · no translation",
    mixed: "Keep Mixed · عربي + English",
  }[route] || "Keep original · no translation";
  select.appendChild(original);
  const displayNames = typeof Intl.DisplayNames === "function"
    ? new Intl.DisplayNames(["en"], { type: "language" })
    : null;
  const orderedCodes = [
    "en",
    "ar",
    ...TRANSLATION_LANGUAGE_CODES.filter((code) => !["en", "ar"].includes(code)),
  ];
  orderedCodes.forEach((code) => {
    const option = document.createElement("option");
    option.value = code;
    option.textContent = `${displayNames?.of(code) || code.toUpperCase()} · ${code}`;
    select.appendChild(option);
  });
}

function renderTranslatorStatus() {
  const progress = state.downloads.get(TRANSLATOR_ID);
  const installed = Boolean(state.translator?.installed);
  elements.translatorCard.classList.toggle("is-ready", installed);
  elements.translatorCard.classList.toggle("is-downloading", Boolean(progress));
  elements.installTranslator.disabled = installed || Boolean(progress) || !invoke;
  elements.installTranslator.textContent = installed
    ? "Installed"
    : progress
      ? `${progress.percent ?? 0}%`
      : "Download 526 MB";
  elements.settingsInstallTranslator.disabled = installed || Boolean(progress) || !invoke;
  elements.settingsInstallTranslator.textContent = installed
    ? "Installed"
    : progress
      ? `${progress.percent ?? 0}%`
      : "Download 526 MB";
  const statusText = installed
    ? "Installed locally · translations never leave this PC"
    : progress
      ? downloadProgressLabel(progress)
      : "Required only when an output language differs from the spoken language.";
  elements.translatorStatus.textContent = statusText;
  elements.settingsTranslatorStatus.textContent = statusText;
}

function activeModel() {
  const modelId = state.routes[state.language];
  return state.models.find((model) => model.id === modelId) || state.models[0];
}

function updateActiveRouteUi() {
  const model = activeModel();
  elements.providerSelect.value = model?.id || "";
  elements.activeRouteLabel.textContent = `Model for ${routeLabels[state.language]}`;
  elements.activeOutputLabel.textContent = `Output for ${routeLabels[state.language]}`;
  elements.outputSelect.dataset.outputRoute = state.language;
  populateOutputSelect(elements.outputSelect, state.language);
  elements.outputSelect.value =
    state.outputRoutes[state.language] || DEFAULT_OUTPUT_ROUTES[state.language];
  elements.activeModelLabel.textContent = model?.name || "No model";
  if (!invoke) {
    updateBrowserPreviewUi();
    return;
  }
  const usesCloud = model?.connection === "api";
  elements.privacyBadge.classList.toggle("is-cloud", usesCloud);
  elements.privacyBadgeText.textContent = usesCloud ? "CLOUD API" : "LOCAL";
  elements.privacyBadge.setAttribute(
    "aria-label",
    usesCloud ? "Cloud API connection" : "Local connection",
  );
  elements.privacyLine.innerHTML = usesCloud
    ? `<span class="privacy-dot" aria-hidden="true"></span>Cloud route · audio is sent to the selected provider`
    : `<span class="privacy-dot" aria-hidden="true"></span>Local route · audio stays on this PC`;
  elements.sessionPrivacyTitle.textContent = usesCloud ? "Cloud route selected" : "Local route";
  elements.sessionPrivacyNote.textContent = usesCloud
    ? `Recordings in ${routeLabels[state.language]} mode are sent to ${model.name}.`
    : "Audio stays on this computer when the selected route is local.";
}

function updateBrowserPreviewUi() {
  elements.privacyBadge.classList.remove("is-cloud");
  elements.privacyBadge.classList.add("is-preview");
  elements.privacyBadgeText.textContent = "PREVIEW";
  elements.privacyBadge.setAttribute("aria-label", "Browser preview only");
  elements.privacyLine.innerHTML =
    `<span class="privacy-dot" aria-hidden="true"></span>Browser preview · speech models are unavailable`;
  elements.sessionPrivacyTitle.textContent = "Preview only";
  elements.sessionPrivacyNote.textContent =
    "Open Start-TypeSpeak.cmd to run real local transcription.";
}

async function startRecording({ fromHotkey = false } = {}) {
  if (state.recording || state.starting || state.processing) return;
  const model = activeModel();
  if (!invoke && model?.backendEngine !== "mock") {
    showPersistentError(
      "Chrome cannot run local speech models. Open Start-TypeSpeak.cmd from the project folder.",
    );
    return;
  }
  if (!model?.configured) {
    if (model?.managed || model?.connection === "managed") {
      await installSpeechModel(model.id);
      return;
    }
    toast(
      "Model setup needed",
      `Configure ${model?.name || "a model"} before using the ${routeLabels[state.language]} route.`,
      true,
    );
    openSettings();
    return;
  }
  if (!navigator.mediaDevices?.getUserMedia) {
    toast("Microphone unavailable", "This runtime does not expose microphone capture.", true);
    return;
  }

  state.starting = true;
  state.recordRequested = true;
  try {
    const microphoneStream = await requestMicrophoneStream();
    if (!state.recordRequested) {
      microphoneStream.getTracks().forEach((track) => track.stop());
      return;
    }
    connectRecorder(microphoneStream);
    activateRecording(fromHotkey);
  } catch (error) {
    state.recordRequested = false;
    toast(
      "Microphone permission needed",
      error?.message || "Allow microphone access in Windows and try again.",
      true,
    );
  } finally {
    state.starting = false;
  }
}

function requestMicrophoneStream() {
  return navigator.mediaDevices.getUserMedia({
    audio: {
      channelCount: 1,
      echoCancellation: true,
      noiseSuppression: true,
      autoGainControl: true,
    },
  });
}

function connectRecorder(microphoneStream) {
  const AudioContext = window.AudioContext || window.webkitAudioContext;
  const audioContext = new AudioContext();
  const source = audioContext.createMediaStreamSource(microphoneStream);
  const processor = audioContext.createScriptProcessor(4096, 1, 1);
  const analyser = audioContext.createAnalyser();
  analyser.fftSize = 256;
  analyser.smoothingTimeConstant = 0.55;
  processor.onaudioprocess = captureAudioSamples;
  source.connect(analyser);
  analyser.connect(processor);
  processor.connect(audioContext.destination);
  Object.assign(state, {
    stream: microphoneStream,
    audioContext,
    source,
    processor,
    analyser,
    sampleRate: audioContext.sampleRate,
    samples: [],
  });
}

function captureAudioSamples(event) {
  if (state.recording) {
    const channel = event.inputBuffer.getChannelData(0);
    state.samples.push(new Float32Array(channel));
  }
}

function activateRecording(fromHotkey) {
  state.startedAt = performance.now();
  state.recording = true;
  state.targetCaptured = fromHotkey;
  state.autoInsert = fromHotkey;
  setStatus("recording", "Listening");
  elements.laneStatus.textContent = "Speak naturally — Arabic and English can mix";
  updateTargetCapture(fromHotkey);
  updateRecordingUi();
  animateWaveform();
  startRecordingTimer();
}

function updateTargetCapture(fromHotkey) {
  elements.targetNote.classList.toggle("is-captured", fromHotkey);
  elements.targetNote.querySelector("strong").textContent = fromHotkey
    ? "Target app captured"
    : "Test recording";
  elements.targetNote.querySelector("p").textContent = fromHotkey
    ? "The final transcript will return to the active text field."
    : `Use ${shortcutLabel()} from another app for automatic insertion.`;
}

function startRecordingTimer() {
  state.timerId = window.setInterval(() => {
    const elapsedMs = performance.now() - state.startedAt;
    elements.recordTimer.textContent = formatTimer(elapsedMs);
    if (elapsedMs >= 90_000) stopRecording();
  }, 80);
}

async function stopRecording() {
  state.recordRequested = false;
  if (!state.recording) return;
  const durationMs = Math.round(performance.now() - state.startedAt);
  state.recording = false;
  await disconnectRecorder(durationMs);
  if (!recordingIsUsable(durationMs)) return;
  beginProcessing();
  await transcribeCapturedAudio(durationMs);
}

async function disconnectRecorder(durationMs) {
  window.clearInterval(state.timerId);
  window.cancelAnimationFrame(state.animationId);
  elements.recordTimer.textContent = formatTimer(durationMs);
  state.processor.onaudioprocess = null;
  state.processor.disconnect();
  state.source.disconnect();
  state.stream.getTracks().forEach((track) => track.stop());
  await state.audioContext.close();
  emitOverlayLevels(Array(9).fill(0));
  updateRecordingUi();
}

function recordingIsUsable(durationMs) {
  if (durationMs >= 650 && state.samples.length > 0) return true;
  resetAfterProcessing();
  toast("Recording too short", "Hold the shortcut and speak for at least one second.", true);
  return false;
}

function beginProcessing() {
  state.processing = true;
  setStatus("processing", "Transcribing");
  elements.laneStatus.textContent = `Running ${activeModel()?.name || "the selected model"}`;
  updateRecordingUi();
  animateProcessingWaveform();
}

async function transcribeCapturedAudio(durationMs) {
  let failureMessage = "";
  try {
    const wavBytes = encodeWav(state.samples, state.sampleRate);
    const audioBase64 = bytesToBase64(wavBytes);
    await runTranscription(audioBase64, durationMs);
  } catch (error) {
    failureMessage = error?.message || String(error);
    toast("Transcription failed", failureMessage, true);
  } finally {
    resetAfterProcessing(failureMessage);
  }
}

function resetAfterProcessing(failureMessage = "") {
  state.processing = false;
  elements.speechLane.classList.remove("is-processing");
  updateRecordingUi();
  resetWaveform();
  if (failureMessage) {
    showPersistentError(failureMessage);
    return;
  }
  setStatus(invoke ? "ready" : "preview", invoke ? "Ready" : "Preview");
  elements.laneStatus.textContent = idleLaneMessage();
}

function showPersistentError(message) {
  setStatus("error", "Action needed");
  elements.laneStatus.textContent = message;
}

function idleLaneMessage() {
  return invoke
    ? "Hold the shortcut from any text field"
    : "Preview only — open Start-TypeSpeak.cmd for real transcription";
}

function animateWaveform() {
  const bars = [...elements.waveform.children];
  const frequencyBins = new Uint8Array(state.analyser.frequencyBinCount);
  const timeDomainSamples = new Uint8Array(state.analyser.fftSize);
  const frame = () => {
    if (!state.recording) return;
    state.analyser.getByteFrequencyData(frequencyBins);
    state.analyser.getByteTimeDomainData(timeDomainSamples);
    const waveformLevels = reactiveAudioBars(
      frequencyBins,
      timeDomainSamples,
      bars.length,
    );
    bars.forEach((bar, index) => {
      const center = 1 - Math.abs(index - bars.length / 2) / (bars.length / 2);
      bar.style.height = `${7 + waveformLevels[index] * (22 + center * 42)}px`;
    });
    const now = performance.now();
    if (now - state.lastOverlayLevelAt >= 40) {
      state.lastOverlayLevelAt = now;
      emitOverlayLevels(reactiveAudioBars(frequencyBins, timeDomainSamples, 9));
    }
    state.animationId = window.requestAnimationFrame(frame);
  };
  frame();
}

function audioBands(frequencyBins, count) {
  return Array.from({ length: count }, (_, bandIndex) => {
    const start = Math.floor((bandIndex / count) * frequencyBins.length);
    const end = Math.max(start + 1, Math.floor(((bandIndex + 1) / count) * frequencyBins.length));
    let peak = 0;
    for (let index = start; index < end; index += 1) {
      peak = Math.max(peak, frequencyBins[index]);
    }
    return Math.min(1, peak / 210);
  });
}

function microphoneVoiceLevel(timeDomainSamples) {
  if (!timeDomainSamples.length) return 0;
  let sumSquares = 0;
  for (const sample of timeDomainSamples) {
    const normalized = (sample - 128) / 128;
    sumSquares += normalized * normalized;
  }
  const rootMeanSquare = Math.sqrt(sumSquares / timeDomainSamples.length);
  return Math.min(1, Math.max(0, (rootMeanSquare - 0.006) * 14));
}

function reactiveAudioBars(frequencyBins, timeDomainSamples, count) {
  const spectralLevels = audioBands(frequencyBins, count);
  const voiceLevel = microphoneVoiceLevel(timeDomainSamples);
  return spectralLevels.map((spectralLevel, index) => {
    const distanceFromCenter =
      Math.abs(index - (count - 1) / 2) / Math.max(1, count / 2);
    const centerProfile = 0.34 + (1 - distanceFromCenter) * 0.66;
    const texture = 0.84 + (index % 3) * 0.08;
    return Math.min(1, Math.max(spectralLevel, voiceLevel * centerProfile * texture));
  });
}

function emitOverlayLevels(levels) {
  const emission = emitTo
    ? emitTo("recording-overlay", "typespeak://audio-level", levels)
    : emit?.("typespeak://audio-level", levels);
  if (!emission) return;
  if (emission?.catch) {
    emission.catch((error) => {
      console.warn("TypeSpeak could not update the recording overlay:", error);
    });
  }
}

function animateProcessingWaveform() {
  [...elements.waveform.children].forEach((bar, index) => {
    bar.style.height = `${10 + ((index * 13) % 36)}px`;
    bar.style.animationDelay = `${index * 25}ms`;
  });
}

function resetWaveform() {
  const bars = [...elements.waveform.children];
  bars.forEach((bar, index) => {
    const distance = Math.abs(index - (bars.length - 1) / 2) / ((bars.length - 1) / 2);
    bar.style.height = `${Math.max(6, 18 - distance * 12)}px`;
    bar.style.animationDelay = "";
  });
}

function encodeWav(chunks, sampleRate) {
  const targetSampleRate = 16_000;
  const pcm = resamplePcm(mergeSamples(chunks), sampleRate, targetSampleRate);
  const buffer = new ArrayBuffer(44 + pcm.length * 2);
  const view = new DataView(buffer);
  writeAscii(view, 0, "RIFF");
  view.setUint32(4, 36 + pcm.length * 2, true);
  writeAscii(view, 8, "WAVE");
  writeAscii(view, 12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, targetSampleRate, true);
  view.setUint32(28, targetSampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  writeAscii(view, 36, "data");
  view.setUint32(40, pcm.length * 2, true);

  let offset = 44;
  for (const sample of pcm) {
    const clipped = Math.max(-1, Math.min(1, sample));
    view.setInt16(offset, clipped < 0 ? clipped * 0x8000 : clipped * 0x7fff, true);
    offset += 2;
  }
  return new Uint8Array(buffer);
}

function mergeSamples(chunks) {
  const length = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const merged = new Float32Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    merged.set(chunk, offset);
    offset += chunk.length;
  }
  return merged;
}

function resamplePcm(input, sourceRate, targetRate) {
  if (sourceRate === targetRate || input.length === 0) return input;
  const outputLength = Math.max(1, Math.round(input.length * targetRate / sourceRate));
  const output = new Float32Array(outputLength);
  const ratio = sourceRate / targetRate;
  if (sourceRate > targetRate) {
    for (let index = 0; index < outputLength; index += 1) {
      const start = Math.floor(index * ratio);
      const end = Math.max(start + 1, Math.min(input.length, Math.floor((index + 1) * ratio)));
      let sum = 0;
      for (let sourceIndex = start; sourceIndex < end; sourceIndex += 1) {
        sum += input[sourceIndex];
      }
      output[index] = sum / (end - start);
    }
    return output;
  }
  for (let index = 0; index < outputLength; index += 1) {
    const position = index * ratio;
    const left = Math.floor(position);
    const right = Math.min(input.length - 1, left + 1);
    const blend = position - left;
    output[index] = input[left] * (1 - blend) + input[right] * blend;
  }
  return output;
}

function writeAscii(view, offset, text) {
  for (let index = 0; index < text.length; index += 1) {
    view.setUint8(offset + index, text.charCodeAt(index));
  }
}

function bytesToBase64(bytes) {
  const chunkSize = 0x8000;
  let binary = "";
  for (let offset = 0; offset < bytes.length; offset += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(offset, offset + chunkSize));
  }
  return btoa(binary);
}

async function runTranscription(audioBase64, durationMs) {
  const model = activeModel();
  if (!model) throw new Error("No model is assigned to this language route.");
  const transcription = await requestModelTranscription(model, audioBase64, durationMs);
  recordTranscriptionRun(model.id, transcription, durationMs);
  if (!transcription.ok) {
    throw new Error(transcription.error || "The selected model did not return a transcript.");
  }
  const cleanedText = await cleanedTranscript(transcription.text);
  const finalText = await translatedTranscript(cleanedText);
  setTranscript(finalText, transcription);
  const targetLanguage = state.outputRoutes[state.language];
  elements.activeModelLabel.textContent = outputPreservesRoute(state.language, targetLanguage)
    ? model.name
    : `${model.name} → ${languageDisplayName(targetLanguage)}`;
  addRecentTranscript(finalText, model, transcription, durationMs);
  await deliverTranscript(finalText, model);
}

function requestModelTranscription(model, audioBase64, durationMs) {
  if (model.connection === "native") {
    return callNative("transcribe_audio", {
      request: {
        engine: model.backendEngine,
        audioBase64,
        language: state.language,
        durationMs,
        glossary: state.glossary,
      },
    });
  }
  if (model.connection === "managed") {
    return callNative("transcribe_managed_model", {
      request: {
        connectionId: model.id,
        fileName: model.localFileName,
        backend: model.backendEngine || "auto",
        audioBase64,
        language: state.language,
        durationMs,
      },
    });
  }
  return callNative("transcribe_model_endpoint", {
    request: endpointRequest(model, audioBase64, durationMs),
  });
}

function endpointRequest(model, audioBase64, durationMs) {
  return {
    connectionId: model.id,
    connection: model.connection,
    endpoint: model.endpoint,
    apiKey: model.connection === "api" ? state.modelSecrets.get(model.id) || null : null,
    model: model.model,
    audioBase64,
    language: state.language,
    durationMs,
  };
}

function recordTranscriptionRun(modelId, transcription, durationMs) {
  state.transcriptions = [transcription];
  state.runs += 1;
  state.totalDurationMs += durationMs;
  state.enginesUsed.add(modelId);
  updateMetrics();
}

async function cleanedTranscript(transcript) {
  let finalText = transcript;
  if (state.cleanupMode === "polished") {
    finalText = await callNative("normalize_transcript", { text: finalText });
  }
  return finalText;
}

async function translatedTranscript(transcript) {
  const targetLanguage =
    state.outputRoutes[state.language] || DEFAULT_OUTPUT_ROUTES[state.language];
  if (targetLanguage === "original") return transcript;
  const sourceLanguage = translationSourceLanguage(transcript, targetLanguage);
  if (sourceLanguage === targetLanguage) return transcript;
  if (!state.translator?.installed) {
    throw new Error(
      "The local translator is not installed. Open Settings and download M2M100.",
    );
  }
  setStatus("processing", "Translating");
  elements.laneStatus.textContent =
    `Translating locally: ${languageDisplayName(sourceLanguage)} → ${languageDisplayName(targetLanguage)}`;
  return callNative("translate_text", {
    text: transcript,
    sourceLanguage,
    targetLanguage,
  });
}

function translationSourceLanguage(transcript, targetLanguage) {
  if (state.language !== "mixed") return state.language;
  const arabicCharacters = (transcript.match(/[\u0600-\u06ff]/g) || []).length;
  const latinCharacters = (transcript.match(/[A-Za-z]/g) || []).length;
  if (targetLanguage === "ar") return latinCharacters > 0 ? "en" : "ar";
  if (targetLanguage === "en") return arabicCharacters > 0 ? "ar" : "en";
  return arabicCharacters >= latinCharacters ? "ar" : "en";
}

function languageDisplayName(code) {
  if (code === "original") return "Original";
  if (typeof Intl.DisplayNames !== "function") return code.toUpperCase();
  return new Intl.DisplayNames(["en"], { type: "language" }).of(code) || code.toUpperCase();
}

async function deliverTranscript(finalText, model) {
  if (state.autoInsert && state.targetCaptured) {
    const insertion = await callNative("insert_text", { text: finalText });
    toast(
      insertion.inserted ? "Inserted" : "Transcript ready",
      insertion.message,
      !insertion.inserted,
    );
    return;
  }
  toast(
    model.connection === "api" ? "Cloud transcript ready" : "Local transcript ready",
    model.connection === "api"
      ? `${model.name} returned the transcript.`
      : `${model.name} finished without uploading audio.`,
  );
}

async function copyTranscript() {
  const text = elements.transcript.value.trim();
  if (!text) {
    toast("Nothing to copy", "Record or enter a transcript first.", true);
    return;
  }
  await navigator.clipboard.writeText(text);
  toast("Copied", "Transcript copied to the clipboard.");
}

async function insertTranscript() {
  const text = elements.transcript.value.trim();
  if (!text) {
    toast("Nothing to insert", "Record or enter a transcript first.", true);
    return;
  }
  const insertion = await callNative("insert_text", { text });
  toast(
    insertion.inserted ? "Inserted" : "Could not insert",
    insertion.message,
    !insertion.inserted,
  );
}

function bindEvents() {
  bindLanguageControls();
  bindTranscriptControls();
  bindRecordingControls();
  bindSettingsControls();
  bindGlossaryControls();
  bindNavigationControls();
  bindRecentControls();
  bindDictionaryControls();
  bindWindowControls();
  bindEscapeKey();
}

function bindLanguageControls() {
  elements.languageControl.addEventListener("click", (event) => {
    const button = event.target.closest("[data-language]");
    if (!button || state.recording || state.processing) return;
    state.language = button.dataset.language;
    elements.languageControl
      .querySelectorAll(".segment")
      .forEach((segment) => segment.classList.toggle("is-selected", segment === button));
    updateActiveRouteUi();
  });
  elements.providerSelect.addEventListener("change", async () => {
    await selectRouteModel(state.language, elements.providerSelect.value);
  });
  routeSelectElements().forEach((select) => {
    select.addEventListener("change", async () => {
      await selectRouteModel(select.dataset.route, select.value);
    });
  });
  elements.outputSelect.addEventListener("change", async () => {
    await selectOutputLanguage(state.language, elements.outputSelect.value);
  });
  outputSelectElements().forEach((select) => {
    select.addEventListener("change", async () => {
      await selectOutputLanguage(select.dataset.outputRoute, select.value);
    });
  });
}

async function selectRouteModel(route, modelId) {
  const previous = state.routes[route];
  const model = state.models.find((candidate) => candidate.id === modelId);
  if (!model) return;
  if (!model.configured) {
    if (model.managed || model.connection === "managed") {
      toast(
        "Download model first",
        `Use the Download button beside ${model.name}, then assign it to this language.`,
        true,
      );
      state.routes[route] = previous;
      renderRouteSelects();
      updateActiveRouteUi();
      return;
    } else {
      toast("Model setup needed", `Configure ${model.name} before assigning it.`, true);
      openModelEditor(model.id);
      state.routes[route] = previous;
      renderRouteSelects();
      updateActiveRouteUi();
      return;
    }
  }
  state.routes[route] = modelId;
  persistRoutes();
  renderRouteSelects();
  updateActiveRouteUi();
}

async function selectOutputLanguage(route, language) {
  if (!["original", ...TRANSLATION_LANGUAGE_CODES].includes(language)) {
    state.outputRoutes[route] = DEFAULT_OUTPUT_ROUTES[route];
    renderOutputSelects();
    updateActiveRouteUi();
    return;
  }
  state.outputRoutes[route] = language;
  persistOutputRoutes();
  renderOutputSelects();
  updateActiveRouteUi();
  if (outputNeedsTranslator(route, language) && !state.translator?.installed) {
    toast(
      "Translator download needed",
      "Download the local translator in Settings before using this output language.",
      true,
    );
  }
}

function outputNeedsTranslator(route, language) {
  return !outputPreservesRoute(route, language);
}

function outputPreservesRoute(route, language) {
  return language === "original"
    || (route === "ar" && language === "ar")
    || (route === "en" && language === "en");
}

function bindTranscriptControls() {
  elements.cleanupMode.addEventListener("click", async (event) => {
    const button = event.target.closest("[data-mode]");
    if (!button) return;
    state.cleanupMode = button.dataset.mode;
    elements.cleanupMode
      .querySelectorAll(".mode-tab")
      .forEach((modeTab) => modeTab.classList.toggle("is-active", modeTab === button));
    if (state.cleanupMode === "polished" && elements.transcript.value.trim()) {
      setTranscript(
        await callNative("normalize_transcript", { text: elements.transcript.value }),
      );
    }
  });
  elements.transcript.addEventListener("input", () => setTranscript(elements.transcript.value));
  elements.clearTranscript.addEventListener("click", () => setTranscript(""));
  elements.copyTranscript.addEventListener("click", copyTranscript);
  elements.insertTranscript.addEventListener("click", insertTranscript);
}

function bindRecordingControls() {
  elements.recordButton.addEventListener("click", () => {
    if (state.recording) stopRecording();
    else startRecording();
  });
  elements.refreshProviders.addEventListener("click", loadEngines);
}

function bindSettingsControls() {
  elements.connectionHelp.addEventListener("click", openSettings);
  elements.openSettings.addEventListener("click", openSettings);
  elements.closeSettings.addEventListener("click", closeSettings);
  bindShortcutSettings();
  bindModelSettings();
  bindRecentSettings();
  bindStartupSettings();
}

function bindStartupSettings() {
  elements.startWithWindowsToggle.addEventListener("change", saveStartupPreference);
}

async function saveStartupPreference() {
  const requested = elements.startWithWindowsToggle.checked;
  elements.startWithWindowsToggle.disabled = true;
  try {
    await callNative("set_startup_enabled", { enabled: requested });
    const enabled = await callNative("startup_enabled");
    applyStartupPreference(enabled);
    showStartupPreferenceToast(enabled);
  } catch (error) {
    elements.startWithWindowsToggle.checked = state.startupEnabled;
    toast("Startup setting unavailable", error?.message || String(error), true);
  } finally {
    elements.startWithWindowsToggle.disabled = false;
  }
}

function applyStartupPreference(enabled) {
  state.startupEnabled = enabled;
  elements.startWithWindowsToggle.checked = enabled;
  localStorage.setItem(STARTUP_PREFERENCE_STORAGE_KEY, String(enabled));
}

function showStartupPreferenceToast(enabled) {
  toast(
    enabled ? "Windows startup enabled" : "Windows startup disabled",
    enabled
      ? "TypeSpeak will launch quietly in the system tray when you sign in."
      : "TypeSpeak will no longer start automatically with Windows.",
  );
}

async function initializeStartupPreference() {
  if (!invoke) {
    state.startupEnabled = true;
    elements.startWithWindowsToggle.checked = true;
    elements.startWithWindowsToggle.disabled = true;
    return;
  }
  const savedPreference = localStorage.getItem(STARTUP_PREFERENCE_STORAGE_KEY);
  try {
    let enabled = await callNative("startup_enabled");
    if (savedPreference === null) {
      await callNative("set_startup_enabled", { enabled: true });
      enabled = await callNative("startup_enabled");
      applyStartupPreference(enabled);
      return;
    }
    state.startupEnabled = enabled;
    elements.startWithWindowsToggle.checked = enabled;
  } catch (error) {
    elements.startWithWindowsToggle.checked = state.startupEnabled;
    elements.startWithWindowsToggle.disabled = true;
    toast("Startup setting unavailable", error?.message || String(error), true);
  }
}

function bindShortcutSettings() {
  elements.changeShortcut.addEventListener("click", toggleShortcutCapture);
  elements.changeShortcutPage.addEventListener("click", toggleShortcutCapture);
  document.querySelectorAll("[data-shortcut-option]").forEach((button) => {
    button.addEventListener("click", () => {
      if (!state.recording && !state.processing) {
        savePushToTalkShortcut(button.dataset.shortcutOption);
      }
    });
  });
  document.addEventListener("keydown", capturePushToTalkShortcut);
  elements.settingsModal.addEventListener("click", (event) => {
    if (event.target === elements.settingsModal) closeSettings();
  });
}

function bindModelSettings() {
  elements.modelForm.addEventListener("submit", saveModelConnection);
  elements.cancelModelEdit.addEventListener("click", resetModelForm);
  elements.modelConnection.addEventListener("change", updateModelFormFields);
  elements.installTranslator.addEventListener("click", installTranslator);
  elements.settingsInstallTranslator.addEventListener("click", installTranslator);
}

function bindRecentSettings() {
  elements.saveRecentToggle.addEventListener("change", () => {
    state.saveRecent = elements.saveRecentToggle.checked;
    localStorage.setItem(SAVE_RECENT_STORAGE_KEY, String(state.saveRecent));
    if (!state.saveRecent) {
      state.recent = [];
      localStorage.removeItem(RECENT_STORAGE_KEY);
    }
    renderRecentHistory();
    toast(
      state.saveRecent ? "Recent enabled" : "Recent disabled",
      state.saveRecent
        ? "Future transcripts will be saved locally."
        : "Saved transcript history was cleared.",
    );
  });
}

function toggleShortcutCapture() {
  state.capturingShortcut = !state.capturingShortcut;
  renderPushToTalkShortcut();
}

async function capturePushToTalkShortcut(event) {
  if (!state.capturingShortcut || event.repeat) return;
  event.preventDefault();
  event.stopImmediatePropagation();
  if (event.key === "Escape") {
    state.capturingShortcut = false;
    renderPushToTalkShortcut();
    return;
  }
  const shortcut = shortcutFromKeyEvent(event);
  if (shortcut === "") return;
  if (!shortcut) {
    toast(
      "Unsupported Windows key",
      "Choose a standard key, F1–F24, navigation, numpad, volume, or media button.",
      true,
    );
    return;
  }
  await savePushToTalkShortcut(shortcut);
}

function shortcutFromKeyEvent(event) {
  if (["Control", "Shift", "Alt", "Meta"].includes(event.key)) return "";
  if (!supportedShortcutCode(event.code)) return null;
  const modifiers = [];
  if (event.ctrlKey) modifiers.push("Control");
  if (event.altKey) modifiers.push("Alt");
  if (event.shiftKey) modifiers.push("Shift");
  if (event.metaKey) modifiers.push("Super");
  return [...modifiers, event.code].join("+");
}

function supportedShortcutCode(code) {
  return SHORTCUT_CODE_PATTERN.test(code);
}

async function savePushToTalkShortcut(shortcut) {
  try {
    await callNative("set_push_to_talk_shortcut", { shortcut });
    state.pushToTalkShortcut = shortcut;
    state.capturingShortcut = false;
    localStorage.setItem(SHORTCUT_STORAGE_KEY, shortcut);
    renderPushToTalkShortcut();
    updateRecordingUi();
    toast(
      "Shortcut changed",
      `Quick tap keeps ${shortcutLabel()} normal. Hold it for 0.3 seconds to talk.`,
    );
  } catch (error) {
    toast("Shortcut unavailable", error?.message || String(error), true);
  }
}

async function activateSavedPushToTalkShortcut() {
  if (!invoke) return;
  try {
    await callNative("set_push_to_talk_shortcut", { shortcut: state.pushToTalkShortcut });
  } catch (error) {
    state.pushToTalkShortcut = DEFAULT_PUSH_TO_TALK_SHORTCUT;
    localStorage.removeItem(SHORTCUT_STORAGE_KEY);
    renderPushToTalkShortcut();
    toast("Saved shortcut reset", error?.message || String(error), true);
  }
}

function bindGlossaryControls() {
  elements.addGlossary.addEventListener("click", addGlossaryTerm);
  elements.glossaryInput.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      addGlossaryTerm();
    }
  });
}

function bindDictionaryControls() {
  elements.dictionaryForm.addEventListener("submit", (event) => {
    event.preventDefault();
    addGlossaryValue(elements.dictionaryInput.value);
    elements.dictionaryInput.value = "";
  });
  elements.dictionarySearch.addEventListener("input", renderDictionary);
}

function bindRecentControls() {
  elements.clearRecent.addEventListener("click", () => {
    state.recent = [];
    persistRecentHistory();
    renderRecentHistory();
    toast("Recent cleared", "Local transcript history was removed.");
  });
}

function bindNavigationControls() {
  document.querySelectorAll("[data-view]").forEach((button) => {
    button.addEventListener("click", () => showView(button.dataset.view));
  });
}

function showView(view) {
  const nextView = APP_VIEWS.has(view) ? view : "dictate";
  document
    .querySelectorAll(".rail-button[data-view]")
    .forEach((button) => button.classList.toggle("is-active", button.dataset.view === nextView));
  document.querySelectorAll("[data-view-panel]").forEach((panel) => {
    panel.classList.toggle("is-active", panel.dataset.viewPanel === nextView);
    if (panel.dataset.viewPanel === nextView) panel.scrollTop = 0;
  });
  if (nextView === "recent") renderRecentHistory();
  if (nextView === "dictionary") renderDictionary();
}

function bindWindowControls() {
  elements.minimizeWindow.addEventListener("click", () => callNative("minimize_window"));
  elements.maximizeWindow.addEventListener("click", () => callNative("toggle_maximize_window"));
  elements.closeWindow.addEventListener("click", () => callNative("close_window"));
}

function bindEscapeKey() {
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      if (!elements.settingsModal.classList.contains("is-hidden")) closeSettings();
      else if (state.recording) stopRecording();
    }
  });
}

function openSettings() {
  elements.settingsModal.classList.remove("is-hidden");
  elements.closeSettings.focus();
}

function closeSettings() {
  state.capturingShortcut = false;
  renderPushToTalkShortcut();
  elements.settingsModal.classList.add("is-hidden");
}

function openModelEditor(modelId) {
  const model = state.models.find((candidate) => candidate.id === modelId);
  if (!model || model.connection === "native") return;
  elements.modelEditId.value = model.id;
  elements.modelName.value = model.name;
  elements.modelConnection.value = model.connection;
  elements.modelEndpoint.value = model.downloadUrl || model.endpoint || "";
  elements.modelIdentifier.value = model.connection === "managed"
    ? model.backendEngine || "auto"
    : model.model || "";
  elements.modelApiKey.value = "";
  elements.modelChecksum.value = model.expectedSha256 || "";
  elements.modelFormTitle.textContent = `Configure ${model.name}`;
  elements.saveModel.textContent = "Save connection";
  elements.cancelModelEdit.classList.remove("is-hidden");
  updateModelFormFields();
  openSettings();
  elements.modelName.focus();
}

function resetModelForm() {
  elements.modelForm.reset();
  elements.modelEditId.value = "";
  elements.modelConnection.value = "managed";
  elements.modelFormTitle.textContent = "Connect another model";
  elements.saveModel.textContent = "Add model";
  elements.cancelModelEdit.classList.add("is-hidden");
  updateModelFormFields();
}

function updateModelFormFields() {
  const connection = elements.modelConnection.value;
  const managed = connection === "managed";
  elements.modelEndpointLabel.textContent = managed
    ? "Hugging Face or direct GGUF URL"
    : "Transcription endpoint";
  elements.modelIdentifierLabel.textContent = managed
    ? "CrispASR backend"
    : "Model ID";
  elements.modelEndpoint.placeholder = managed
    ? "https://huggingface.co/org/repo/resolve/main/model.gguf"
    : connection === "api"
      ? "https://provider.example/v1/audio/transcriptions"
      : "http://127.0.0.1:8000/v1/audio/transcriptions";
  elements.modelIdentifier.placeholder = managed
    ? "auto, cohere, qwen3, omniasr…"
    : "organization/model-name";
  elements.modelApiKeyField.classList.toggle("is-hidden", managed);
  elements.modelChecksumField.classList.toggle("is-hidden", !managed);
  elements.modelApiKey.required = connection === "api";
  elements.modelFormNote.textContent = managed
    ? "Paste a Hugging Face file link or direct GGUF URL. TypeSpeak downloads and verifies it locally."
    : "OpenAI-compatible multipart endpoint returning a JSON text field.";
}

async function saveModelConnection(event) {
  event.preventDefault();
  const editId = elements.modelEditId.value;
  const connection = elements.modelConnection.value;
  const apiKey = elements.modelApiKey.value.trim();
  const endpointError = connectionEndpointError(connection, elements.modelEndpoint.value);
  if (endpointError) {
    toast("Invalid endpoint", endpointError, true);
    elements.modelEndpoint.focus();
    return;
  }
  if (missingApiKey(connection, apiKey, editId)) {
    toast("API key needed", "Enter an API key for this cloud connection.", true);
    elements.modelApiKey.focus();
    return;
  }
  const existing = state.models.find((model) => model.id === editId);
  const nextModel = modelFromForm(existing);
  rememberModelSecret(nextModel.id, connection, apiKey);
  upsertModel(existing, nextModel);
  persistModels();
  renderEngines();
  resetModelForm();
  if (connection === "managed") {
    toast(
      "Model added",
      `${nextModel.name} is ready. Use its Download button when you want to install it locally.`,
    );
  } else {
    toast("Model connected", `${nextModel.name} is now available in all three routes.`);
  }
}

function connectionEndpointError(connection, endpoint) {
  let url;
  try {
    url = new URL(endpoint);
  } catch {
    return "Enter a valid HTTP or HTTPS transcription URL.";
  }
  if (connection === "managed") {
    if (url.protocol !== "https:") return "Managed model downloads must use HTTPS.";
    if (!decodeURIComponent(url.pathname).toLowerCase().endsWith(".gguf")) {
      return "The managed model URL must point to a .gguf file.";
    }
    return "";
  }
  if (connection === "api" && url.protocol !== "https:") {
    return "Cloud API connections must use HTTPS.";
  }
  const loopbackHosts = new Set(["localhost", "127.0.0.1", "[::1]"]);
  if (connection === "local" && !loopbackHosts.has(url.hostname)) {
    return "Local endpoints must use localhost, 127.0.0.1, or ::1.";
  }
  return "";
}

function missingApiKey(connection, apiKey, editId) {
  return connection === "api" && !apiKey && !state.modelSecrets.has(editId);
}

function modelFromForm(existing) {
  const connection = elements.modelConnection.value;
  return connection === "managed"
    ? managedModelFromForm(existing)
    : endpointModelFromForm(existing, connection);
}

function managedModelFromForm(existing) {
  const id = existing?.id || `custom-${Date.now().toString(36)}`;
  const downloadUrl = normalizeManagedDownloadUrl(elements.modelEndpoint.value.trim());
  return {
    ...(existing || {}),
    id,
    name: elements.modelName.value.trim(),
    connection: "managed",
    downloadUrl,
    localFileName: managedFileName(downloadUrl, id),
    backendEngine: elements.modelIdentifier.value.trim() || "auto",
    model: elements.modelIdentifier.value.trim() || "auto",
    expectedSha256: elements.modelChecksum.value.trim() || null,
    configured: false,
    managed: true,
    builtIn: existing?.builtIn || false,
    note: "User-managed GGUF model. Runs locally through the bundled CrispASR runtime.",
    setupHint: "Ready to download",
  };
}

function endpointModelFromForm(existing, connection) {
  const usesCloud = connection === "api";
  return {
    ...(existing || {}),
    id: existing?.id || `custom-${Date.now().toString(36)}`,
    name: elements.modelName.value.trim(),
    connection,
    endpoint: elements.modelEndpoint.value.trim(),
    model: elements.modelIdentifier.value.trim(),
    configured: true,
    builtIn: existing?.builtIn || false,
    note: usesCloud
      ? "Custom cloud transcription endpoint. Audio is uploaded."
      : "Custom local transcription endpoint.",
    setupHint: usesCloud ? "API endpoint configured" : "Local endpoint configured",
  };
}

function normalizeManagedDownloadUrl(downloadUrl) {
  const url = new URL(downloadUrl);
  if (url.hostname.toLowerCase() === "huggingface.co" && url.pathname.includes("/blob/")) {
    url.pathname = url.pathname.replace("/blob/", "/resolve/");
  }
  return url.toString();
}

function managedFileName(downloadUrl, modelId) {
  const pathname = decodeURIComponent(new URL(downloadUrl).pathname);
  const sourceName = pathname.split("/").filter(Boolean).pop();
  const safeSource = sourceName.replace(/[^A-Za-z0-9._-]/g, "-");
  const stem = safeSource.slice(0, -5);
  const prefix = `${modelId}-`;
  const maxStemLength = Math.max(1, 160 - prefix.length - ".gguf".length);
  return `${prefix}${stem.slice(0, maxStemLength)}.gguf`;
}

function rememberModelSecret(modelId, connection, apiKey) {
  if (connection === "api" && apiKey) {
    state.modelSecrets.set(modelId, apiKey);
  } else if (connection !== "api") {
    state.modelSecrets.delete(modelId);
  }
}

function upsertModel(existing, nextModel) {
  if (existing) {
    state.models = state.models.map((model) =>
      model.id === nextModel.id ? nextModel : model);
  } else {
    state.models.push(nextModel);
  }
}

function removeModel(modelId) {
  const model = state.models.find((candidate) => candidate.id === modelId);
  if (!model || model.builtIn) return;
  state.models = state.models.filter((candidate) => candidate.id !== modelId);
  state.modelSecrets.delete(modelId);
  for (const route of Object.keys(state.routes)) {
    if (state.routes[route] === modelId) state.routes[route] = "whisper-local";
  }
  persistModels();
  persistRoutes();
  renderEngines();
  toast("Model removed", `${model.name} was removed from TypeSpeak.`);
}

async function bindGlobalHotkey() {
  if (!listen) return;
  await listen("typespeak://hotkey", async (event) => {
    if (event.payload === "pressed") {
      await startRecording({ fromHotkey: true });
    } else if (event.payload === "released") {
      await stopRecording();
    }
  });
}

async function bindTrayNavigation() {
  if (!listen) return;
  await listen("typespeak://navigate", (event) => {
    showView(String(event.payload || "dictate"));
  });
}

async function initialize() {
  state.routes = loadSavedRoutes();
  state.outputRoutes = loadSavedOutputRoutes();
  state.pushToTalkShortcut = loadSavedShortcut();
  state.glossary = loadSavedGlossary();
  state.recent = loadRecentHistory();
  state.saveRecent = loadSaveRecentPreference();
  elements.saveRecentToggle.checked = state.saveRecent;
  await initializeStartupPreference();
  repairOutputRoutes();
  createWaveform();
  renderGlossary();
  renderRecentHistory();
  renderPushToTalkShortcut();
  bindEvents();
  updateModelFormFields();
  await bindModelDownloadProgress();
  await loadEngines();
  await bindGlobalHotkey();
  await bindTrayNavigation();
  await activateSavedPushToTalkShortcut();
  updateRecordingUi();
  updateMetrics();
  setTranscript("");
  installRequestedDefaultModel().catch((error) => {
    console.warn("TypeSpeak could not start the installer-requested model download:", error);
  });

  if (!invoke) {
    setStatus("preview", "Preview");
    elements.laneStatus.textContent = idleLaneMessage();
    toast(
      "Browser preview only",
      "Open Start-TypeSpeak.cmd from the project folder to run real speech models.",
    );
  }
}

initialize();
