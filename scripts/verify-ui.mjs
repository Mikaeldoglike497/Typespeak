import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const [html, app, css, overlayHtml, overlayScript] = await Promise.all([
  readFile(new URL("../src/index.html", import.meta.url), "utf8"),
  readFile(new URL("../src/app.js", import.meta.url), "utf8"),
  readFile(new URL("../src/styles.css", import.meta.url), "utf8"),
  readFile(new URL("../src/overlay.html", import.meta.url), "utf8"),
  readFile(new URL("../src/overlay.js", import.meta.url), "utf8"),
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

console.log(
  `UI contract passed: ${ids.length} unique ids, ${panels.length} views, 9 live overlay bars.`,
);
