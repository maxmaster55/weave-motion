/*!
 * weave.js: spring motion for the web. One file, no build step, no wasm required.
 * A plain JavaScript port of weaver-motion (same solver, presets and reduced-motion policy).
 * MIT OR Apache-2.0, https://github.com/maxmaster55/weave-motion
 *
 * Simple API
 *   const x = Weave.spring(0, v => el.style.left = v + 'px');   // create
 *   x.to(300);                                                  // animate, retarget any time
 *   await Weave.animate('.card', { y: 0, opacity: 1 }, { from: { y: 20, opacity: 0 } });
 *
 * Works as a <script> tag (global `Weave`), with require() and with import (via bundlers / Node).
 */
(function (root, factory) {
  if (typeof module === 'object' && module.exports) module.exports = factory();
  else root.Weave = factory();
})(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';

  /* ===================================================================== *
   *  Core: a line-by-line port of weaver-motion/src/spring.rs, tokens.rs and *
   *  policy.rs. Keep it in sync; js/test checks it against Rust samples.  *
   * ===================================================================== */

  const CRITICAL_TOLERANCE = 1e-4;
  const DEFAULT_POSITION_EPSILON = 0.01;
  const DEFAULT_VELOCITY_EPSILON = 0.01;

  /** The four spatial presets of weaver-motion (MotionToken). [stiffness, damping ratio] */
  const PRESETS = {
    fast: [500, 0.85],        // fast_spatial: quick feedback for direct manipulation
    default: [300, 0.8],      // default_spatial: everyday movement
    expressive: [220, 0.65],  // expressive_spatial: visible, friendly bounce
    gentle: [140, 1.0],       // gentle_spatial: large surfaces, no overshoot
  };

  function presetName(name) {
    if (typeof name !== 'string') return 'default';
    const short = name.replace(/_spatial$/, '');
    return PRESETS[short] ? short : 'default';
  }

  /** SpringConfig::new, validated like the Rust version. */
  function springConfig(mass, stiffness, dampingRatio) {
    if (!(Number.isFinite(mass) && mass > 0)) throw new RangeError('weave: mass must be a positive number');
    if (!(Number.isFinite(stiffness) && stiffness > 0)) throw new RangeError('weave: stiffness must be a positive number');
    if (!(Number.isFinite(dampingRatio) && dampingRatio >= 0)) throw new RangeError('weave: damping ratio must be >= 0');
    return { mass, stiffness, dampingRatio, positionEpsilon: DEFAULT_POSITION_EPSILON, velocityEpsilon: DEFAULT_VELOCITY_EPSILON };
  }

  function presetConfig(name) {
    const [k, z] = PRESETS[presetName(name)];
    return springConfig(1, k, z);
  }

  /** MotionPolicy::spring_config. policy: 'full' | 'reduced' | 'off' */
  function applyPolicy(base, policy) {
    if (policy === 'reduced') return Object.assign({}, base, { dampingRatio: 1, stiffness: base.stiffness * 2 });
    if (policy === 'off') return Object.assign({}, base, { dampingRatio: 1, stiffness: base.stiffness * 100 });
    return base;
  }

  /**
   * Closed-form damped spring (weaver_motion::solve). Returns [value, velocity] after `dt`
   * seconds. The result does not depend on how `dt` is split into steps.
   */
  function solve(config, value, velocity, target, dt) {
    if (!Number.isFinite(dt) || dt <= 0) return [value, velocity];
    if (!Number.isFinite(value) || !Number.isFinite(velocity) || !Number.isFinite(target)) return [value, velocity];

    const omega = Math.sqrt(config.stiffness / config.mass);
    const zeta = config.dampingRatio;
    const y0 = value - target;
    const v0 = velocity;
    let y, v;

    if (zeta < 1 - CRITICAL_TOLERANCE) {
      // Underdamped: oscillates around the target.
      const alpha = zeta * omega;
      const omegaD = omega * Math.sqrt((1 - zeta) * (1 + zeta));
      const a = y0;
      const b = (v0 + alpha * y0) / omegaD;
      const decay = Math.exp(-alpha * dt);
      const sin = Math.sin(omegaD * dt), cos = Math.cos(omegaD * dt);
      y = decay * (a * cos + b * sin);
      v = decay * ((b * omegaD - alpha * a) * cos + (-a * omegaD - alpha * b) * sin);
    } else if (zeta <= 1 + CRITICAL_TOLERANCE) {
      // Critically damped: fastest approach without overshoot.
      const b = v0 + omega * y0;
      const decay = Math.exp(-omega * dt);
      y = (y0 + b * dt) * decay;
      v = (b - omega * (y0 + b * dt)) * decay;
    } else {
      // Overdamped: two real exponentials, no overshoot.
      const root = Math.sqrt((zeta - 1) * (zeta + 1));
      const r1 = -omega * (zeta - root);
      const r2 = -omega * (zeta + root);
      const c1 = (v0 - r2 * y0) / (r1 - r2);
      const c2 = y0 - c1;
      const e1 = Math.exp(r1 * dt), e2 = Math.exp(r2 * dt);
      y = c1 * e1 + c2 * e2;
      v = r1 * c1 * e1 + r2 * c2 * e2;
    }
    return [target + y, v];
  }

  /** weaver_motion::Spring. A value driven by a damped spring; retargeting keeps velocity. */
  class Spring {
    constructor(value, config, target = value, velocity = 0) {
      this.value = value; this.velocity = velocity; this.target = target; this.config = config;
    }
    setTarget(target) { if (Number.isFinite(target)) this.target = target; }
    snap(value) { if (Number.isFinite(value)) { this.value = value; this.target = value; this.velocity = 0; } }
    settle() { this.value = this.target; this.velocity = 0; }
    isSettled() {
      return Math.abs(this.value - this.target) <= this.config.positionEpsilon &&
        Math.abs(this.velocity) <= this.config.velocityEpsilon;
    }
    isActive() { return !this.isSettled(); }
    /** Advances by dt seconds; returns true while still moving. */
    advance(dt) {
      if (this.isSettled()) { this.settle(); return false; }
      if (!Number.isFinite(dt) || dt <= 0) return true;
      const [value, velocity] = solve(this.config, this.value, this.velocity, this.target, dt);
      if (!Number.isFinite(value) || !Number.isFinite(velocity)) { this.settle(); return false; }
      this.value = value; this.velocity = velocity;
      if (this.isSettled()) { this.settle(); return false; }
      return true;
    }
  }

  /**
   * Handle-based spring set. Same API as the wasm `Motion` exported by weave-web,
   * so either can drive the simple API (see Weave.useEngine).
   */
  class Motion {
    constructor(reduced = false) { this._s = []; this._free = []; this._policy = reduced ? 'reduced' : 'full'; }
    setReduced(reduced) {
      this._policy = reduced ? 'reduced' : 'full';
      for (const e of this._s) if (e) e.spring.config = applyPolicy(e.base, this._policy);
    }
    spring(value, preset) {
      const base = presetConfig(preset);
      const e = { spring: new Spring(value, applyPolicy(base, this._policy)), base };
      if (this._free.length) { const id = this._free.pop(); this._s[id] = e; return id; }
      this._s.push(e); return this._s.length - 1;
    }
    release(id) { if (this._s[id]) { this._s[id] = null; this._free.push(id); } }
    setTarget(id, target) { const e = this._s[id]; if (e) e.spring.setTarget(target); }
    snap(id, value) { const e = this._s[id]; if (e) e.spring.snap(value); }
    value(id) { const e = this._s[id]; return e ? e.spring.value : 0; }
    velocity(id) { const e = this._s[id]; return e ? e.spring.velocity : 0; }
    target(id) { const e = this._s[id]; return e ? e.spring.target : 0; }
    isActive(id) { const e = this._s[id]; return !!e && e.spring.isActive(); }
    advance(dt) {
      dt = Math.min(Math.max(dt, 0), 0.1);
      let active = false;
      for (const e of this._s) if (e && e.spring.advance(dt)) active = true;
      return active;
    }
  }

  /* ===================================================================== *
   *  Simple API: springs with callbacks on one shared frame clock.        *
   * ===================================================================== */

  const hasWindow = typeof window !== 'undefined';
  const mq = hasWindow && window.matchMedia ? window.matchMedia('(prefers-reduced-motion: reduce)') : null;
  let reducedSetting = 'auto';              // true | false | 'auto'
  const isReduced = () => (reducedSetting === 'auto' ? !!(mq && mq.matches) : !!reducedSetting);

  let engine = new Motion(isReduced());
  const live = new Set();                   // springs with something to do
  const timers = [];                        // delayed starts, counted in engine time
  let manual = !hasWindow || typeof requestAnimationFrame !== 'function';
  let rafId = 0, last = 0;

  function frame(now) {
    rafId = 0;
    const dt = last ? (now - last) / 1000 : 1 / 60;
    last = now;
    tick(dt);
    if (live.size || timers.length) rafId = requestAnimationFrame(frame);
    else last = 0;
  }
  function wake() { if (!manual && !rafId && (live.size || timers.length)) rafId = requestAnimationFrame(frame); }

  /** Runs `fn` after `seconds` of engine time (so delays follow Weave.manual / Weave.tick too). */
  function after(seconds, fn) {
    if (!(seconds > 0)) { fn(); return; }
    timers.push({ left: seconds, fn });
    wake();
  }

  /** Advances every spring by `dt` seconds. Called automatically each frame unless `Weave.manual(true)`. */
  function tick(dt) {
    for (const tm of timers.slice()) {
      tm.left -= dt;
      if (tm.left <= 0) { timers.splice(timers.indexOf(tm), 1); tm.fn(); }
    }
    engine.advance(dt);
    for (const s of Array.from(live)) s._step();
  }

  class SimpleSpring {
    constructor(value, onUpdate, preset) {
      this._preset = presetName(preset);
      this._id = engine.spring(value, this._preset);
      this._subs = onUpdate ? [onUpdate] : [];
      this._waiters = [];
      this._last = value;
    }
    /** Current value. */
    get value() { return engine.value(this._id); }
    /** Where it is going. */
    get target() { return engine.target(this._id); }
    /** Current speed (units per second). */
    get velocity() { return engine.velocity(this._id); }
    /** True while moving. */
    get moving() { return engine.isActive(this._id); }
    /** Animates to `target`. Calling again mid-flight keeps the momentum. Resolves when it comes to rest. */
    to(target) {
      engine.setTarget(this._id, Number(target));
      live.add(this); wake();
      return new Promise(resolve => this._waiters.push(resolve));
    }
    /** Jumps to `value` without animating. */
    set(value) {
      engine.snap(this._id, Number(value));
      this._emit(); this._rest();
      return this;
    }
    /** Adds a listener called with every new value. */
    onChange(fn) { this._subs.push(fn); return this; }
    /** Stops where it is now. */
    stop() { engine.snap(this._id, this.value); this._rest(); return this; }
    /** Frees the spring. */
    destroy() { this.stop(); engine.release(this._id); this._subs = []; all.delete(this); }
    _emit() {
      const v = this.value;
      if (v === this._last) return;
      this._last = v;
      for (const fn of this._subs) fn(v);
    }
    _step() {
      this._emit();
      if (!engine.isActive(this._id)) this._rest();
    }
    _rest() {
      live.delete(this);
      const w = this._waiters; this._waiters = [];
      for (const resolve of w) resolve(this.value);
    }
    _rebind(next) {   // move to another engine, keeping value and target
      const v = this.value, t = this.target;
      this._id = next.spring(v, this._preset);
      next.setTarget(this._id, t);
    }
  }

  const all = new Set();

  /**
   * Creates a spring.
   *   Weave.spring(0)                                 // value only
   *   Weave.spring(0, v => ...)                       // with a listener
   *   Weave.spring(0, v => ..., { preset: 'expressive' })
   *   Weave.spring(0, { preset: 'fast', onChange: v => ... })
   */
  function spring(value = 0, onChange, options) {
    if (onChange && typeof onChange === 'object') { options = onChange; onChange = options.onChange; }
    const s = new SimpleSpring(Number(value), onChange, options && options.preset);
    all.add(s);
    return s;
  }

  /* ---------------- element animation ---------------- */

  const DEFAULTS = { x: 0, y: 0, scale: 1, rotate: 0, opacity: 1 };
  const states = new WeakMap();

  function elements(target) {
    if (!target) return [];
    if (typeof target === 'string') return hasWindow ? Array.from(document.querySelectorAll(target)) : [];
    if (typeof target.length === 'number' && !target.nodeType) return Array.from(target);
    return [target];
  }

  function stateOf(el, preset) {
    let st = states.get(el);
    if (!st) {
      st = { props: {} };
      states.set(el, st);
      const opacity = hasWindow ? parseFloat(getComputedStyle(el).opacity) : 1;
      st.initialOpacity = Number.isFinite(opacity) ? opacity : 1;
    }
    for (const k of Object.keys(DEFAULTS)) {
      if (!st.props[k]) {
        const start = k === 'opacity' ? st.initialOpacity : DEFAULTS[k];
        st.props[k] = spring(start, () => render(el, st), { preset });
      }
    }
    return st;
  }

  function render(el, st) {
    const p = k => st.props[k].value;
    el.style.transform = `translate(${p('x')}px, ${p('y')}px) rotate(${p('rotate')}deg) scale(${p('scale')})`;
    el.style.opacity = String(Math.min(1, Math.max(0, p('opacity'))));
  }

  /**
   * Animates elements' position, scale, rotation and opacity with springs.
   *   Weave.animate(el, { x: 100 })                                  // move right 100px
   *   Weave.animate('.card', { y: 0, opacity: 1 }, { from: { y: 24, opacity: 0 }, stagger: 0.05 })
   * props: x, y (px), scale, rotate (deg), opacity.  options: preset, from, stagger (s), delay (s).
   * Returns a Promise that resolves when everything is at rest.
   */
  function animate(target, props, options = {}) {
    const els = elements(target);
    const jobs = els.map((el, i) => {
      const st = stateOf(el, options.preset);
      if (options.from) for (const [k, v] of Object.entries(options.from)) if (st.props[k]) st.props[k].set(v);
      render(el, st);
      const run = () => Promise.all(Object.entries(props).filter(([k]) => st.props[k]).map(([k, v]) => st.props[k].to(v)));
      const delay = (options.delay || 0) + (options.stagger || 0) * i;
      return new Promise(r => after(isReduced() ? 0 : delay, () => run().then(r)));
    });
    return Promise.all(jobs).then(() => undefined);
  }

  /** Fades and slides elements in: Weave.enter('.item', { stagger: 0.04 }). */
  function enter(target, options = {}) {
    return animate(target, { y: 0, opacity: 1, scale: 1 }, Object.assign({ from: { y: options.distance ?? 16, opacity: 0, scale: 1 } }, options));
  }

  /* ---------------- settings ---------------- */

  /** true / false to force, 'auto' (default) to follow the operating system setting. */
  function reducedMotion(value) {
    reducedSetting = value === true || value === false ? value : 'auto';
    engine.setReduced(isReduced());
  }
  if (mq && mq.addEventListener) mq.addEventListener('change', () => { if (reducedSetting === 'auto') engine.setReduced(isReduced()); });

  /** Drive time yourself (tests, video export): Weave.manual(true); Weave.tick(1/60). */
  function setManual(on) { manual = !!on; if (manual && rafId && typeof cancelAnimationFrame === 'function') { cancelAnimationFrame(rafId); rafId = 0; } wake(); }

  /**
   * Uses another engine with the Motion API, e.g. the wasm build of weave-web:
   *   import init, { Motion } from './weave_web.js'; await init(); Weave.useEngine(new Motion(false));
   */
  function useEngine(next) {
    for (const s of all) s._rebind(next);
    engine = next;
    engine.setReduced(isReduced());
    return engine;
  }

  return {
    // simple API
    spring, animate, enter, after, tick, reducedMotion, manual: setManual, useEngine,
    isReducedMotion: isReduced,
    // core (same as weaver-motion)
    solve, Spring, Motion, springConfig, presetConfig, applyPolicy,
    PRESETS: Object.freeze(Object.keys(PRESETS)),
    version: '0.1.0',
  };
});
