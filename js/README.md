# weave.js

Spring animation for any web page or app. One file, no build step, no dependencies.
It runs the same physics as the Rust crates (`weaver-motion`), in plain JavaScript.
The wasm build is optional.

## Add it

Pick one:

```html
<!-- any page: copy weave.js next to your HTML -->
<script src="weave.js"></script>
```

```js
// Node, bundlers (Vite, webpack, …), Electron, Tauri
const Weave = require('./weave.js');   // or: import Weave from './weave.js'
```

TypeScript types are in `weave.d.ts`.

## Use it

### Move an element

```js
Weave.animate('#box', { x: 200 });                    // slide right 200 px
Weave.animate('#box', { scale: 1.2, rotate: 10 });    // grow and tilt
Weave.animate('#box', { opacity: 0 });                // fade out
```

`x`, `y` are pixels, `rotate` is degrees, `scale` and `opacity` are plain numbers.
Calling `animate` again while it moves changes direction smoothly.

### Make things appear

```js
Weave.enter('.card');                       // fade + slide in
Weave.enter('.card', { stagger: 0.05 });    // one after another
```

### Animate any number

```js
const width = Weave.spring(0, v => bar.style.width = v + '%');
width.to(75);                 // animate to 75
width.to(20);                 // change your mind at any time, momentum is kept
await width.to(100);          // wait until it stops
width.set(0);                 // jump, no animation
```

The callback receives every new value. Use it for anything: text, canvas, WebGL, SVG, a 3D camera.

```js
const temp = Weave.spring(21.4, v => label.textContent = v.toFixed(1) + ' °C');
temp.to(35.2);
```

### Choose the feel

| preset | feel |
|---|---|
| `'fast'` | quick, tight; for things the user drags or presses |
| `'default'` | everyday movement |
| `'expressive'` | a friendly bounce |
| `'gentle'` | slow and smooth, no bounce; for large surfaces |

```js
Weave.animate('#box', { x: 200 }, { preset: 'expressive' });
Weave.spring(0, v => …, { preset: 'fast' });
```

### Accessibility

Weave follows the user's "reduce motion" system setting automatically: no bounce, quicker settle.
Override it with `Weave.reducedMotion(true | false | 'auto')`.

## All functions

| | |
|---|---|
| `Weave.spring(value, onChange?, { preset }?)` | a number that animates. Returns `{ to, set, value, target, velocity, moving, onChange, stop, destroy }` |
| `Weave.animate(target, props, options?)` | animate `x y scale rotate opacity` of an element, a CSS selector or a list. Options: `preset`, `from`, `delay`, `stagger` (seconds). Returns a Promise |
| `Weave.enter(target, options?)` | fade and slide in. Options as above plus `distance` (px) |
| `Weave.after(seconds, fn)` | run something later on the animation clock |
| `Weave.reducedMotion(value)` | `true`, `false` or `'auto'` |
| `Weave.manual(true)` + `Weave.tick(dt)` | drive time yourself: tests, video export |
| `Weave.useEngine(engine)` | swap the engine, e.g. for the wasm build (below) |

## Optional: the wasm engine

The default engine is JavaScript and produces the same numbers as Rust (checked sample by sample in `test/`).
To run the Rust engine instead, build `weave-web` and hand it to Weave. The rest of your code does not change.

```sh
cargo build -p weave-web --features web --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/release/weave_web.wasm
```

```js
import init, { Motion } from './pkg/weave_web.js';
await init();
Weave.useEngine(new Motion(Weave.isReducedMotion()));
```

## Low-level API

`new Weave.Motion(reduced)` is the handle API shared with wasm (`spring`, `setTarget`, `value`, `advance`, …).
`Weave.solve`, `Weave.Spring` and `Weave.presetConfig` mirror `weaver-motion`.

## Tests

```sh
node --test js/test       # or: make js-test
```

`test/fixtures/springs.json` comes from the Rust solver (`cargo run -p weaver-motion --example js_fixture`).
The JS engine must match it sample by sample.
