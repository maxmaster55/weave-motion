// Deterministic samples for the README GIFs, produced by the public JS API.
// node js/tools/samples.js > samples.json   (used by tools/render_js_examples.py)
const fs = require('node:fs');
const path = require('node:path');
const Weave = require('../weave.js');

Weave.manual(true);
Weave.reducedMotion(false);
const FPS = 30, FRAMES = 150, dt = 1 / FPS;

// 1. Toggle switch, including a change of mind mid-flight.
const knob = Weave.spring(0, { preset: 'expressive' });
// 2. A list entering with Weave.enter + stagger (plain objects stand in for DOM elements).
const cards = Array.from({ length: 5 }, () => ({ style: {} }));
// 3. A reading rolling to new values.
const reading = Weave.spring(21.4);
// 4. The same spring in JavaScript vs the Rust solver (fixture written by weave-core).
const rust = JSON.parse(fs.readFileSync(path.join(__dirname, '../test/fixtures/springs.json'), 'utf8'))
  .filter(c => c.policy === 'full' && c.value === 0 && c.target === 1);

const out = { toggle: [], cards: [], reading: [], parity: {} };
const parse = el => {
  const m = /translate\(([-\d.e]+)px, ([-\d.e]+)px\)/.exec(el.style.transform || '');
  return [m ? +m[2] : 16, el.style.opacity === undefined ? 0 : +el.style.opacity];
};
for (let f = 0; f < FRAMES; f++) {
  if (f === 10) knob.to(1);
  if (f === 52) knob.to(0);
  if (f === 58) knob.to(1);        // interrupted: momentum carries through
  if (f === 105) knob.to(0);
  if (f === 8) Weave.enter(cards, { stagger: 0.12, distance: 40 });
  if (f === 95) Weave.animate(cards, { y: 40, opacity: 0 }, { stagger: 0.06, preset: 'fast' });
  if (f === 12) reading.to(35.2);
  if (f === 82) reading.to(24.6);
  Weave.tick(dt);
  out.toggle.push(knob.value);
  out.cards.push(cards.map(parse));
  out.reading.push(reading.value);
}
for (const c of rust) {
  const s = new Weave.Spring(0, Weave.presetConfig(c.preset), 1, 0);
  const js = c.samples.map(() => { s.advance(1 / 60); return s.value; });
  out.parity[c.preset] = { rust: c.samples.map(x => x[0]), js };
}
process.stdout.write(JSON.stringify(out));
