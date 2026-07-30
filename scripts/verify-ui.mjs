import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const [
  html,
  app,
  css,
  overlayHtml,
  overlayScript,
  readme,
  tauriConfig,
  nsisHooks,
  nsisTemplate,
  rustMain,
  rustLib,
  providers,
] = await Promise.all([
  readFile(new URL("../src/index.html", import.meta.url), "utf8"),
  readFile(new URL("../src/app.js", import.meta.url), "utf8"),
  readFile(new URL("../src/styles.css", import.meta.url), "utf8"),
  readFile(new URL("../src/overlay.html", import.meta.url), "utf8"),
  readFile(new URL("../src/overlay.js", import.meta.url), "utf8"),
  readFile(new URL("../README.md", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/windows/hooks.nsh", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/windows/installer-template.nsi", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/src/main.rs", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/src/providers.rs", import.meta.url), "utf8"),
]);

const ids = [...html.matchAll(/\sid="([^"]+)"/g)].map((match) => match[1]);
const duplicateIds = ids.filter((id, index) => ids.indexOf(id) !== index);
assert.deepEqual(duplicateIds, [], `Duplicate HTML ids: ${duplicateIds.join(", ")}`);

const selectedIds = [...app.matchAll(/querySelector\("#([^"]+)"\)/g)].map(
  (match) => match[1],
);
const missingIds = [...new Set(selectedIds)].filter((id) => !ids.includes(id));
assert.deepEqual(missingIds, [], `JavaScript selectors missing from HTML: ${missingIds.join(", ")}`);

const panels = [...html.matchAll(/data-view-panel="([^"]+)"/g)].map((match) => match[1]);
assert.deepEqual(
  panels.sort(),
  ["dictate", "dictionary", "recent", "settings", "shortcuts"],
  "Every production view must have exactly one panel.",
);

const navigationViews = [...html.matchAll(/data-view="([^"]+)"/g)].map(
  (match) => match[1],
);
const unknownViews = navigationViews.filter((view) => !panels.includes(view));
assert.deepEqual(unknownViews, [], `Navigation points to missing panels: ${unknownViews.join(", ")}`);

for (const selector of [
  ".view-panel:not(.is-active)",
  ".page-view.is-active",
  ".recent-row",
  ".dictionary-term",
  ".shortcut-feature-card",
  ".setting-row",
]) {
  assert.ok(css.includes(selector), `Missing production UI style: ${selector}`);
}

assert.equal(
  (overlayHtml.match(/<span(?:\s|>)/g) || []).length,
  9,
  "The recording overlay must contain nine audio-reactive bars.",
);
assert.ok(overlayHtml.includes('src="./overlay.js"'), "The overlay script is not loaded.");
assert.ok(
  overlayScript.includes('listen("typespeak://audio-level"'),
  "The overlay is not listening for microphone levels.",
);
assert.ok(
  app.includes('emitTo("recording-overlay", "typespeak://audio-level"'),
  "Microphone levels must target the recording overlay window.",
);
assert.ok(
  app.includes("getByteTimeDomainData(timeDomainSamples)") &&
    app.includes("microphoneVoiceLevel(timeDomainSamples)"),
  "The recording overlay must react to the live microphone signal.",
);
assert.match(
  providers,
  /fn whisper_language[\s\S]*"ar"\s*=>\s*"ar"[\s\S]*"en"\s*=>\s*"en"[\s\S]*_\s*=>\s*"auto"/,
  "The mixed Whisper route must use automatic language detection.",
);
assert.ok(
  providers.includes("mixed_wav_chunks(job.audio)") &&
    providers.includes("mixed_silence_boundaries"),
  "Mixed Whisper recordings must detect languages independently across safe silence boundaries.",
);
assert.match(
  html,
  /<img class="titlebar-icon" src="\.\/assets\/typespeak-icon\.png" alt="" \/>/,
  "The title bar must use the official TypeSpeak icon.",
);
assert.ok(!html.includes("brand-glyph"), "The legacy title-bar glyph must not remain.");
assert.match(
  html,
  /<link rel="icon" type="image\/png" href="\.\/assets\/typespeak-icon\.png" \/>/,
  "The app document must use the official TypeSpeak icon.",
);
assert.equal(
  (html.match(/id="settingsOutput(?:Arabic|Mixed|English)"/g) || []).length,
  3,
  "Settings must expose independent Arabic, Mixed, and English output-language controls.",
);
assert.ok(
  app.includes('original.textContent = {') &&
    app.includes('"Keep Mixed · عربي + English"'),
  "Output controls must clearly distinguish preserving the transcript from translation.",
);
assert.match(
  html,
  /<a class="developer-credit" href="https:\/\/nabilnet\.ai"[^>]*>Developed by <b>NABILNET\.AI<\/b><\/a>/,
  "The app must visibly credit NABILNET.AI with a working link.",
);
assert.match(
  readme,
  /Developed by \[NABILNET\.AI\]\(https:\/\/nabilnet\.ai\)/,
  "The README must credit NABILNET.AI with a working link.",
);
assert.ok(!/speakly|research/i.test(readme), "The README must not mention Speakly or research.");
assert.ok(!tauriConfig.includes('"../models/"'), "Speech models must not be bundled.");
assert.ok(
  tauriConfig.includes('"version": "0.1.3"') &&
    tauriConfig.includes('"icons/icon.ico"') &&
    tauriConfig.includes('"installerIcon": "icons/icon.ico"') &&
    tauriConfig.includes('"template": "./windows/installer-template.nsi"'),
  "The Windows app and installer must use the current TypeSpeak icon and release version.",
);
assert.ok(
  nsisTemplate.includes(
    "!insertmacro TYPESPEAK_STOP_BACKGROUND_SERVICES typespeak_upgrade_stop_retry",
  ) &&
    nsisTemplate.includes('StrCpy $R1 "$R1 /S /UPDATE"') &&
    nsisTemplate.includes("typespeak_upgrade_repair_retry") &&
    nsisTemplate.includes('Delete "$INSTDIR\\${MAINBINARYNAME}.exe"'),
  "NSIS upgrades must stop background services, uninstall silently, and repair broken legacy installs.",
);
assert.match(
  html,
  /id="startWithWindowsToggle"[^>]*type="checkbox"[^>]*checked/,
  "Settings must expose Windows startup as an enabled-by-default option.",
);
assert.ok(
  rustLib.includes("tauri_plugin_autostart::init") &&
    rustLib.includes("set_startup_enabled") &&
    rustLib.includes("WINDOWS_STARTUP_ARGUMENT") &&
    tauriConfig.includes('"visible": false'),
  "Windows startup must use Tauri autostart and launch quietly in the system tray.",
);
assert.match(
  nsisHooks,
  /Download the default Whisper model now\?[\s\S]*--download-default-model/,
  "The NSIS installer must offer the default model download.",
);
assert.match(
  nsisHooks,
  /NSIS_HOOK_PREINSTALL[\s\S]*TYPESPEAK_STOP_BACKGROUND_SERVICES typespeak_preinstall_stop_retry/,
  "The NSIS installer must invoke native background-service shutdown.",
);
assert.match(
  nsisHooks,
  /NSIS_HOOK_PREUNINSTALL[\s\S]*TYPESPEAK_STOP_BACKGROUND_SERVICES typespeak_preuninstall_stop_retry/,
  "The NSIS uninstaller must invoke the verified background-service shutdown helper.",
);
assert.ok(
  nsisHooks.includes('KillProcessCurrentUser "${executable_name}"') &&
    nsisHooks.includes(
      'TYPESPEAK_VERIFY_CURRENT_USER_PROCESS_STOPPED "whisper-server.exe"',
    ),
  "NSIS must use Tauri's native plugin to stop and verify the local Whisper service.",
);
assert.ok(
  !app.includes("window.confirm"),
  "Model downloads must not use a blocking WebView confirmation dialog.",
);
assert.match(
  rustMain,
  /windows_subsystem\s*=\s*"windows"/,
  "The Windows app must launch without a console window.",
);
assert.ok(
  rustLib.includes("window.set_icon(icon.clone())") &&
    rustLib.includes("tray = tray.icon(icon.clone())"),
  "The main window, taskbar, and tray must all use the bundled TypeSpeak icon.",
);
assert.match(
  providers,
  /creation_flags\(CREATE_NO_WINDOW\)/,
  "Local model processes must stay hidden on Windows.",
);

console.log(
  `UI contract passed: ${ids.length} unique ids, ${panels.length} views, 9 live overlay bars.`,
);
