const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const source = fs.readFileSync('src-tauri/src/player.js', 'utf8');

function player({ host = 'www.tiktok.com', videos = [], button = null, scrollingElement = null, shadowRoot = null } = {}) {
  const notices = [];
  const context = {
    window: {},
    location: { protocol: 'https:', hostname: host }, innerHeight: 800,
    setTimeout: () => 1, clearTimeout: () => {},
    document: {
      scrollingElement,
      querySelectorAll: selector => selector === '*' ? shadowRoot ? [{shadowRoot}] : [] : selector === 'video' ? videos : button ? [button] : [],
      getElementById: () => null,
      createElement: () => ({ style: {}, set textContent(value) { notices.push(value); } }),
      documentElement: { appendChild() {} },
    },
  };
  return { run: action => vm.runInNewContext(`(${source})(${JSON.stringify(action)})`, context), notices };
}
function video(top = 0) {
  return { paused: false, volume: 0.95, muted: true,
    getBoundingClientRect: () => ({ top, bottom: top + 600, width: 300, height: 600 }),
    pause() { this.paused = true; }, play() { this.paused = false; return Promise.resolve(); },
  };
}
test('pause and resume the visible video without keyboard events', () => {
  const v = video(); const p = player({ videos: [v] });
  p.run('pause'); assert.equal(v.paused, true);
  p.run('pause'); assert.equal(v.paused, false);
});
test('volume is clamped and unmutes playback', () => {
  const v = video(); const p = player({ videos: [v] });
  p.run('louder'); assert.equal(v.volume, 1); assert.equal(v.muted, false);
  v.volume = 0.05; p.run('quieter'); assert.equal(v.volume, 0);
  p.run('mute'); assert.equal(v.muted, true);
});
test('selects the video closest to the viewport center, ignores offscreen videos', () => {
  const offscreen = video(-900), above = video(-400), current = video(100);
  player({ videos: [offscreen, above, current] }).run('pause');
  assert.equal(current.paused, true); assert.equal(above.paused, false); assert.equal(offscreen.paused, false);
});
test('uses the site navigation button when available', () => {
  let clicks = 0;
  const button = { getBoundingClientRect: video().getBoundingClientRect, click() { clicks++; } };
  player({ button }).run('next'); assert.equal(clicks, 1);
});
test('reports unsupported pages and missing players', () => {
  const p = player(); p.run('next'); p.run('pause');
  assert.equal(p.notices.length, 2);
});
test('scrolls the page in either direction when feed arrows are absent', () => {
  const scrolls = [];
  const scrollingElement = { scrollHeight: 3000, clientHeight: 800, scrollBy: options => scrolls.push(options.top) };
  const p = player({ scrollingElement });
  p.run('next'); p.run('previous');
  assert.deepEqual(scrolls, [680, -680]);
});
test('does not control lookalike domains or other websites', () => {
  for (const host of ['example.com', 'tiktok.com.evil.test', 'eviltiktok.com']) {
    const v = video(); player({ host, videos: [v] }).run('pause'); assert.equal(v.paused, false);
  }
});
test('finds a video inside a shadow root', () => {
  const v = video();
  const shadowRoot = {querySelectorAll: selector => selector === 'video' ? [v] : []};
  player({shadowRoot}).run('pause'); assert.equal(v.paused, true);
});
test('recognizes the French next-video button', () => {
  let clicks = 0;
  const button = {disabled: false, getBoundingClientRect: video().getBoundingClientRect, click() { clicks++; }};
  const shadowRoot = {querySelectorAll: selector => selector.includes('Aller à la vidéo suivante') ? [button] : []};
  player({shadowRoot}).run('next'); assert.equal(clicks, 1);
});
test('clicks the player pause control even when its video is encapsulated', () => {
  let clicks = 0;
  const button = {getBoundingClientRect: video().getBoundingClientRect, click() { clicks++; }};
  const shadowRoot = {querySelectorAll: selector => selector.includes('button[aria-label="Pause"]') ? [button] : []};
  player({shadowRoot}).run('pause'); assert.equal(clicks, 1);
});
