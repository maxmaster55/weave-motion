// Run: node --test js/test
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Weave = require('../weave.js');

const close = (a, b, tol = 1e-9) => Math.abs(a - b) <= tol * Math.max(1, Math.abs(b));

test('matches weave-core (Rust) sample by sample', () => {
  const cases = JSON.parse(fs.readFileSync(path.join(__dirname, 'fixtures/springs.json'), 'utf8'));
  assert.equal(cases.length, 36);
  for (const c of cases) {
    const config = Weave.applyPolicy(Weave.presetConfig(c.preset), c.policy);
    const s = new Weave.Spring(c.value, config, c.target, c.velocity);
    c.samples.forEach(([value, velocity, moving], i) => {
      const m = s.advance(1 / 60);
      const where = `${c.preset}/${c.policy} from ${c.value} sample ${i}`;
      assert.ok(close(s.value, value), `${where}: value ${s.value} != ${value}`);
      assert.ok(close(s.velocity, velocity), `${where}: velocity ${s.velocity} != ${velocity}`);
      assert.equal(m, moving, `${where}: moving`);
    });
    const [v, vel] = Weave.solve(config, c.value, c.velocity, c.target, 0.75);
    assert.ok(close(v, c.big[0]) && close(vel, c.big[1]), `${c.preset}/${c.policy}: large step`);
  }
});

test('solver is independent of the step count', () => {
  const cfg = Weave.presetConfig('expressive');
  const one = Weave.solve(cfg, 0, 0, 1, 0.5);
  let state = [0, 0];
  for (let i = 0; i < 50; i++) state = Weave.solve(cfg, state[0], state[1], 1, 0.01);
  assert.ok(close(state[0], one[0], 1e-9) && close(state[1], one[1], 1e-9));
});

test('Motion: same handle API as the wasm build', () => {
  const m = new Weave.Motion(false);
  const id = m.spring(0, 'default');
  m.setTarget(id, 100);
  for (let i = 0; i < 1200 && m.advance(1 / 60); i++);
  assert.equal(m.value(id), 100);
  assert.equal(m.isActive(id), false);

  m.setTarget(id, 0); m.advance(1 / 60);
  const v = m.velocity(id); m.setTarget(id, 50);
  assert.equal(m.velocity(id), v, 'retargeting keeps velocity');

  m.release(id); m.release(id);
  assert.equal(m.spring(2, 'gentle_spatial'), id, 'handles are reused');
  assert.equal(m.value(99), 0); assert.equal(m.isActive(99), false);
});

test('reduced motion never overshoots', () => {
  const m = new Weave.Motion(true);
  const id = m.spring(0, 'expressive');
  m.setTarget(id, 1);
  let max = 0;
  for (let i = 0; i < 600; i++) { m.advance(1 / 60); max = Math.max(max, m.value(id)); }
  assert.ok(max <= 1 + 1e-9);
});

test('simple API: spring().to() animates, notifies and resolves at rest', async () => {
  Weave.manual(true);
  Weave.reducedMotion(false);
  const seen = [];
  const x = Weave.spring(0, v => seen.push(v));
  const done = x.to(10);
  assert.equal(x.moving, true);
  for (let i = 0; i < 600 && x.moving; i++) Weave.tick(1 / 60);
  assert.equal(await done, 10);
  assert.equal(x.value, 10);
  assert.ok(seen.length > 10 && seen.at(-1) === 10);
});

test('simple API: retarget mid-flight keeps momentum; set() jumps', () => {
  Weave.manual(true);
  const x = Weave.spring(0, { preset: 'expressive' });
  x.to(1);
  for (let i = 0; i < 6; i++) Weave.tick(1 / 60);
  const v = x.velocity;
  x.to(0);
  assert.equal(x.velocity, v);
  x.set(5);
  assert.equal(x.value, 5); assert.equal(x.moving, false);
});

test('simple API: animate() moves an element with transform + opacity', async () => {
  Weave.manual(true);
  const el = { style: {} };
  const done = Weave.animate(el, { x: 120, opacity: 1 }, { from: { opacity: 0 } });
  assert.match(el.style.transform, /translate\(0px, 0px\)/);
  assert.equal(el.style.opacity, '0');
  for (let i = 0; i < 600; i++) Weave.tick(1 / 60);
  await done;
  assert.equal(el.style.transform, 'translate(120px, 0px) rotate(0deg) scale(1)');
  assert.equal(el.style.opacity, '1');
});

test('simple API: useEngine() swaps the backend and keeps state', () => {
  Weave.manual(true);
  const x = Weave.spring(3);
  x.to(7);
  Weave.useEngine(new Weave.Motion(false));   // stands in for the wasm Motion
  assert.equal(x.value, 3); assert.equal(x.target, 7);
  for (let i = 0; i < 600; i++) Weave.tick(1 / 60);
  assert.equal(x.value, 7);
});

test('simple API: stagger delays run on the engine clock', async () => {
  Weave.manual(true);
  const els = [{ style: {} }, { style: {} }, { style: {} }];
  const done = Weave.enter(els, { stagger: 0.1 });
  Weave.tick(0.05);   // only the first has started
  const y = el => Number(/translate\(0px, ([-\d.e]+)px\)/.exec(el.style.transform)[1]);
  assert.ok(y(els[0]) < 16 && y(els[2]) === 16);
  for (let i = 0; i < 900; i++) Weave.tick(1 / 60);
  await done;
  assert.ok(els.every(el => el.style.opacity === '1' && y(el) === 0));
});
