const listen = window.__TAURI__?.event?.listen;
const bars = [...document.querySelectorAll(".listening-pill span")];
const targetLevels = bars.map(() => 0);
const displayLevels = bars.map(() => 0);
let lastAudioFrameAt = 0;

function clampLevel(value) {
  return Math.max(0, Math.min(1, Number(value) || 0));
}

function renderAudioFrame(timestamp) {
  const audioIsFresh = timestamp - lastAudioFrameAt < 180;
  bars.forEach((bar, index) => {
    const target = audioIsFresh ? targetLevels[index] : 0;
    const smoothing = target > displayLevels[index] ? 0.52 : 0.2;
    displayLevels[index] += (target - displayLevels[index]) * smoothing;
    const centerWeight = 1 - Math.abs(index - (bars.length - 1) / 2) / (bars.length / 2);
    const height = 4 + displayLevels[index] * (10 + centerWeight * 9);
    bar.style.height = `${height.toFixed(1)}px`;
    bar.style.opacity = String(0.52 + displayLevels[index] * 0.48);
  });
  window.requestAnimationFrame(renderAudioFrame);
}

async function bindAudioLevels() {
  if (!listen) return;
  await listen("typespeak://audio-level", (event) => {
    const levels = Array.isArray(event.payload) ? event.payload : [];
    targetLevels.forEach((_, index) => {
      targetLevels[index] = clampLevel(levels[index]);
    });
    lastAudioFrameAt = performance.now();
  });
}

bindAudioLevels().catch((error) => {
  console.error("TypeSpeak could not listen for microphone levels:", error);
});
window.requestAnimationFrame(renderAudioFrame);
