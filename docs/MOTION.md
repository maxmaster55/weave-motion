# Motion guide

This is the motion guide for every product built on `weaver-motion`: the desktop shell,
web dashboards of live AI-agent tiles, and widgets. It is theme-independent. A theme
such as the pixel-art Glowgrid may change how a role is *drawn* (snapping to a grid,
stepping effects, its own art cadence). It may not change what a role *means*, the
reduced-motion mapping, the live-data rules or the accessibility limits in this guide.

Each rule carries a tag for its evidence:

| Tag | Meaning |
|---|---|
| **S** | Strong: a peer-reviewed experiment or a normative standard (WCAG, CSS spec). |
| **C** | Convention: what the major design systems ship, cited. Most millisecond values are C. |
| **I** | Inference: derived by the research from S and C sources. Tune it if you have data. |

Background: the research report `UI motion design principles.md` (in the agent_gui
project, `reports/`) and its notes, which hold the computations behind the numbers here.
Sources are linked inline and collected at the end.

## Contents

1. [Principles](#1-principles)
2. [Choosing a role](#2-choosing-a-role)
3. [Spring tokens](#3-spring-tokens)
4. [Tweens: durations and when to use them](#4-tweens-durations-and-when-to-use-them)
5. [Distances and scales](#5-distances-and-scales)
6. [Stagger and ordering](#6-stagger-and-ordering)
7. [Interruption and retargeting](#7-interruption-and-retargeting)
8. [Live data](#8-live-data)
9. [Reduced motion](#9-reduced-motion)
10. [Accessibility limits](#10-accessibility-limits)
11. [Performance](#11-performance)
12. [Testing motion deterministically](#12-testing-motion-deterministically)
13. [Converting between parameter styles](#13-converting-between-parameter-styles)
14. [What themes may change](#14-what-themes-may-change)
15. [Sources](#15-sources)

## 1. Principles

1. **Motion is a budget of onsets.** The *start* of a motion captures attention
   automatically, even when the motion carries no information
   ([Abrams & Christ 2003](https://pubmed.ncbi.nlm.nih.gov/12930472/)). On a surface
   that is always visible, every onset is a withdrawal from the user's attention. Spend
   it on state changes worth interrupting someone for. **S**
2. **Routine data never moves things.** A progress tick, a new log line or a token count
   changes text, colour or opacity. Movement, scale and pops are reserved for an agent
   needing input, failing or finishing. **S** (derived from onset capture)
3. **Continuous motion is the most expensive kind.** Continuously scrolling tickers
   distracted more than start-stop ones and were remembered no better
   ([Maglio & Campbell 2000](https://www.interruptions.net/literature/Maglio-CHI00-p241-maglio.pdf)).
   Loops must be informative, and pausable. **S**
4. **Re-layouts animate, all together.** Animated transitions help people keep track of
   objects through a data change ([Heer & Robertson 2007](https://idl.cs.washington.edu/files/2007-AnimatedTransitions-InfoVis.pdf)).
   Staggering them does not help tracking and can hurt it
   ([Chevalier et al. 2014](https://hal.archives-ouvertes.fr/hal-01054408v1)). **S**
5. **Springs for anything spatial or interruptible; tweens for sequenced, looping or
   data-bound change.** A curve has no way to take an initial velocity, so it "jerks to a
   halt" when interrupted ([WWDC23](https://developer.apple.com/videos/play/wwdc2023/10158/)). **C**
6. **Calm by default.** Start with zero bounce; the standard scheme is the default and
   expressive is opt-in for rare, user-initiated moments. **C**
7. **Start now.** Feedback begins on the next frame and is visibly underway within about
   70–100 ms; perceived causality fades beyond about 70 ms
   ([Michotte, via Psychonomic](https://link.springer.com/content/pdf/10.3758%2FBF03194815.pdf)),
   and 0.1 s is the limit for "instantaneous"
   ([NN/g](https://www.nngroup.com/articles/response-times-3-important-limits/)).
   Every spring in this guide reaches 30% of its travel within 72 ms. **S/I**
8. **Stay inside 100–500 ms.** That is the band every system agrees on; 300 ms is the
   ceiling for ordinary UI tweens ([Primer](https://github.com/primer/primitives/blob/main/src/tokens/functional/motion/motion.json5),
   [NN/g](https://www.nngroup.com/articles/animation-duration/)). **C**
9. **Never restart.** A changed destination keeps the current position and velocity. **C**
10. **Reduced motion substitutes; it does not delete.** **C**
11. **Idle at rest.** Nothing draws, ticks or wakes the CPU when nothing is changing. **C**

## 2. Choosing a role

Pick the role by *what changes* and *how big it is*, never by how it should feel. Size
selects the tier because a spring takes the same time for 10 px as for 1000 px
([MDC Motion](https://github.com/material-components/material-components-android/blob/master/docs/theming/Motion.md)).

| What changes | Role | Examples | Tag |
|---|---|---|---|
| A small part moves, or direct feedback | `SpatialFast` | Press, toggle thumb, chip, tab indicator, gesture release | C |
| An element moves or resizes within a surface | `SpatialDefault` | Tile reorder or resize, panels, popovers, menus | C |
| A whole surface or window moves | `SpatialSlow` | Window or full-surface transitions, tile → detail view | C |
| Opacity or colour of a small part | `EffectsFast` | Hover and press tint, focus ring | C |
| Opacity or colour, everyday | `EffectsDefault` | Fades in and out, status colour, crossfades under reduced motion | C |
| Opacity or colour of a large area | `EffectsSlow` | Scrims, background dimming | C |
| The pointer or finger drives it | 1:1 tracking, then `SpatialFast` on release | Drag, swipe, resize handles | C |

```rust
use weaver_motion::{MotionEngine, MotionRole};

let mut engine = MotionEngine::new(); // standard scheme, full motion
let x = engine.spring_for_role("tile.42.x", 0.0, MotionRole::SpatialDefault);
engine.retarget(x, 240.0);
```

**Schemes.** `MotionScheme::standard()` is the default (`MotionScheme::default()`) and the
right choice for the shell, dashboards and widgets. `MotionScheme::expressive()` is opt-in
per surface, for completion, onboarding and similar rare moments the user started. Its only
visible bounce is in `SpatialFast` (9.5% overshoot); the rest of it is mainly slower. Carbon
splits productive from expressive the same way, and Google's own research found a "strong
minority" preferring calmer motion and said the style may not suit "a banking interface"
([Carbon](https://carbondesignsystem.com/elements/motion/overview/),
[Google](https://design.google/library/expressive-material-design-google-research)). **C/I**
`MotionScheme::calm()` (critically damped everywhere) is for surfaces that must never bounce.

**Bounce.** Above bounce 0.3 (ζ < 0.7) only for momentum gestures; Apple calls more than
about 0.4 "too exaggerated for a UI element" ([WWDC23](https://developer.apple.com/videos/play/wwdc2023/10158/)). **C**
Scale uses ζ ≥ 0.9, because overshoot applies to every edge of the element. **I**

**Old tokens.** `MotionToken` presets are superseded. `MotionToken::role()` and
`replacement_spring()` give the replacement: `fast_spatial` → `SpatialDefault`,
`default_spatial` → `SpatialSlow`, `expressive_spatial` → expressive `SpatialDefault`,
`gentle_spatial` → only non-interactive background dimming (`EffectsSlow`), `effects` →
`EffectsDefault`. The old values run 281–531 ms perceptual and settle in 391–780 ms. **C/I**

## 3. Spring tokens

Springs are unit mass, stored as stiffness `k` and damping ratio `ζ`. The values are
Material 3's published tokens ([StandardMotionTokens.kt](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/StandardMotionTokens.kt),
[ExpressiveMotionTokens.kt](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ExpressiveMotionTokens.kt)). **C**

| Role | Scheme | k / ζ | Perceptual duration | Overshoot | 30% of travel | 2% settle | 1e-3 settle |
|---|---|---|---:|---:|---:|---:|---:|
| `SpatialFast` | standard | 1400 / 0.9 | 168 ms | 0.15% | 28 ms | 126 ms | 224 ms |
| `SpatialDefault` | standard | 700 / 0.9 | 237 ms | 0.15% | 40 ms | 178 ms | 317 ms |
| `SpatialSlow` | standard | 300 / 0.9 | 363 ms | 0.15% | 61 ms | 271 ms | 484 ms |
| `SpatialFast` | expressive | 800 / 0.6 | 222 ms | 9.5% | 34 ms | 210 ms | 359 ms |
| `SpatialDefault` | expressive | 380 / 0.8 | 322 ms | 1.5% | 52 ms | 193 ms | 435 ms |
| `SpatialSlow` | expressive | 200 / 0.8 | 444 ms | 1.5% | 72 ms | 266 ms | 599 ms |
| `EffectsFast` | all | 3800 / 1.0 | 102 ms | 0 | 18 ms | 95 ms | 150 ms |
| `EffectsDefault` | all | 1600 / 1.0 | 157 ms | 0 | 27 ms | 146 ms | 231 ms |
| `EffectsSlow` | all | 800 / 1.0 | 222 ms | 0 | 39 ms | 206 ms | 326 ms |

All columns are computed from the closed form for a step from rest.

- **Perceptual duration** is Apple's `d = 2π/√k` (`SpringConfig::perceptual_duration()`).
  It is stable across damping ratios, so key UI logic (callbacks, the next step of a
  sequence) off it. Apple: "you shouldn't wait for the settling duration for user-facing
  changes" ([WWDC23](https://developer.apple.com/videos/play/wwdc2023/10158/)). **C**
- **Overshoot** is `exp(−πζ/√(1−ζ²))` (`SpringConfig::overshoot()`). Bounce is not
  overshoot: bounce 0.15 overshoots 0.6%, bounce 0.3 overshoots 4.6%.
- **Settle** is the last time the value is outside the band (`SpringConfig::settle_time(ε)`;
  Apple's `settlingDuration` uses ε = 0.001). Use it for cleanup, not for UI logic.

**Settle thresholds are per unit.** "Done" means the remaining change is invisible. **I**,
following [Compose](https://github.com/androidx/androidx/blob/androidx-main/compose/animation/animation-core/src/commonMain/kotlin/androidx/compose/animation/core/VisibilityThresholds.kt)
and [Motion](https://github.com/motiondivision/motion/blob/main/packages/motion-dom/src/animation/generators/spring.ts).

| Unit | Threshold | Constant |
|---|---|---|
| Position or size, px | 0.5 px | `SETTLE_POSITION_PX` |
| Scale factor | 0.001 | `SETTLE_SCALE` |
| Opacity | 1/255 | `SETTLE_OPACITY` |
| Colour channel (0–1) | 1/255 | `SETTLE_COLOR` |

Apply one with `config.with_settle_threshold(SETTLE_SCALE)?`. It sets the position
tolerance to the threshold and the velocity tolerance to threshold × ω, so the motion left
in a settled spring is also below the threshold. Effects roles already use 1/255
(`EFFECTS_EPSILON`), which stops them at about 125 / 193 / 272 ms instead of 150 / 231 /
326 ms. Spatial roles drive both pixels and scale factors, so they keep the default 0.01
units: set `SETTLE_SCALE` yourself when a spatial spring animates scale.

**Stiffness ceiling.** A bouncy spring needs a period of at least 6–8 frames at the lowest
frame rate you support (k ≲ 2300–4000 at 60 Hz); otherwise it reads as jitter. Very high
stiffness is fine only at ζ = 1. **I**

**Own springs.** Prefer a role. If you must tune, describe the spring as a designer would:
`SpringConfig::from_duration_bounce(0.3, 0.0)?`, and check `overshoot()` and
`settle_time(1e-3)` against this table. Damping ratios below 0.05 are evaluated as 0.05
(`MIN_DAMPING_RATIO`), so no spring oscillates forever.

## 4. Tweens: durations and when to use them

Use a fixed-duration tween only when the motion must end at a known time or stay in sync:
choreographed sequences, loops, staggered entrances, progress and other data tweens,
counters, and paths that only CSS or the compositor can run. Tweens cannot take a velocity,
so never use them for anything a user can interrupt mid-flight. **C**

| Token | Duration | Use | Tag |
|---|---:|---|---|
| micro | 100 ms | Hover, press tint, small state changes | C |
| short | 200 ms | Fades, toggles, small enters and exits | C |
| medium | 300 ms | Panels and popovers on CSS-only paths. The UI maximum. | C |
| long | 400–500 ms | Sparingly: large surfaces, first-load choreography | C |

From [Primer](https://github.com/primer/primitives/blob/main/src/tokens/functional/motion/motion.json5)
and [NN/g](https://www.nngroup.com/articles/animation-duration/).

- Quantise durations to 25 ms ([Atlassian](https://unpkg.com/@atlaskit/motion/dist/es2019/utils/durations.js)). **C**
- Curves: decelerate to enter, accelerate to exit, standard for elements that leave but stay
  nearby ([Carbon](https://carbondesignsystem.com/elements/motion/overview/)). Material 3:
  standard `(0.2, 0, 0, 1)`, standard decelerate `(0, 0, 0, 1)`, standard accelerate
  `(0.3, 0, 1, 1)` ([MotionTokens.kt](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/MotionTokens.kt)),
  as `Easing::CubicBezier`. **C**
- Linear only for spinners, progress and colour; Apple warns of "a sharp corner and a jump
  in velocity" at both ends ([WWDC23](https://developer.apple.com/videos/play/wwdc2023/10158/)). **C**
- Exits may be shorter than enters, or symmetric; both are shipped (NN/g versus
  [Fluent](https://github.com/microsoft/fluentui/blob/master/packages/react-components/react-motion-components-preview/library/src/components/Fade/Fade.ts)). **C**
- Duration grows sub-linearly with distance: `clamp(base + a·√distance, 100, 400 ms)`.
  libadwaita clamps swipe completion to the same 100–400 ms
  ([adw-swipe-tracker.c](https://github.com/GNOME/libadwaita/blob/main/src/adw-swipe-tracker.c)).
  For springs, distance selects the role tier instead. **I**
- Carbon's 700 ms values are for dimming and hero transitions, not UI elements. **C**

## 5. Distances and scales

| Quantity | Value | Tag |
|---|---|---|
| Entrance offset | Small, about 30 px or less ([Chrome](https://developer.chrome.com/docs/web-platform/view-transitions/same-document)) | C |
| Entrance and exit scale | 0.85–0.96. Fluent 0.9 / 0.85 for dialogs, Carbon 0.96 for popovers ([Scale.ts](https://github.com/microsoft/fluentui/blob/master/packages/react-components/react-motion-components-preview/library/src/components/Scale/Scale.ts), [DialogSurfaceMotion.ts](https://github.com/microsoft/fluentui/blob/master/packages/react-components/react-dialog/library/src/components/DialogSurfaceMotion.ts), [surfaces.tokens.json](https://github.com/carbon-design-system/carbon/blob/main/packages/motion/tokens/surfaces.tokens.json)) | C |
| Press deformation | At most 15%, M3's button-group expansion ([ButtonGroup.kt](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ButtonGroup.kt)); a 4% squish is well inside | C |
| Completion pop | One event, no loop, scale amplitude ≤ about 5% | I |
| Gesture hysteresis | About 10 pt (iOS) or 16 px (libadwaita) before a drag starts | C |
| Crossfade threshold under reduced motion | Travel over about one tile (`CROSSFADE_DISTANCE_PX` = 96 px), or any zoom to full window | I |

Opening a tile into a detail view is Carbon's "expand" surface: card or tile to side panel,
240 ms, productive standard easing ([surfaces.tokens.json](https://github.com/carbon-design-system/carbon/blob/main/packages/motion/tokens/surfaces.tokens.json)).
With springs, use `SpatialSlow`. **C**

Layout motion is transform-based (FLIP: measure first and last, invert with a transform,
play; [Aerotwist](https://aerotwist.com/blog/flip-your-animations/)). Size changes animate a
scale, so counter-scale or re-lay out text and children, and animate radius and shadow
explicitly. Use shared bounds rather than a shared element when the content differs
([Compose](https://developer.android.com/develop/ui/compose/animation/shared-elements),
[Motion layout](https://motion.dev/docs/react-layout-animations)). **C**

## 6. Stagger and ordering

| Rule | Tag |
|---|---|
| Interval = `min(30 ms, 500 ms ÷ n)` for a group of n items | C ([Carbon](https://carbondesignsystem.com/elements/motion/choreography/): 20 ms within 500 ms) |
| At most 8 items get their own start; later items start with the eighth. The spread is at most 210 ms. | I |
| Stagger only first load and user-triggered bulk adds | C/I |
| Never stagger re-sorts | S ([Chevalier et al.](https://hal.archives-ouvertes.fr/hal-01054408v1)) |
| Never stagger routine live updates, menus or command palettes | C |
| No nested cascades: children enter as one group after their parent, inside the same 500 ms budget | I |
| Order: chrome, then static body, then dynamic data, then primary action, then data visualisation | C ([Carbon](https://carbondesignsystem.com/elements/motion/choreography/)) |
| Exits before moves, moves before enters | C |
| Under reduced motion, a cascade becomes one group fade with no stagger | I |

The tile entrance chain this replaces (each tile starting when the previous one is 30% in)
grows without bound: 12 tiles took 1.0–1.9 s. Use time, not progress.

```rust
use weaver_motion::{stagger_delays, TweenConfig};

let delays = stagger_delays(tiles.len()); // seconds; at most 8 distinct values
for (tile, delay) in tiles.iter().zip(delays) {
    tile.fade_in(TweenConfig::new(0.2).with_delay(delay));
}
```

## 7. Interruption and retargeting

- **Retarget, never queue.** A new destination becomes the spring's equilibrium; the
  current *evaluated* position and velocity are the initial condition (`set_target`,
  `MotionEngine::retarget`). Rebuilding from the model value instead makes velocity
  discontinuous ([objc.io](https://www.objc.io/issues/12-animations/interactive-animations/)). **C**
- **Velocity is in the property's units**: px/s for position, 1/s for scale or progress.
  Convert gesture velocity before handing it over (`Animated::set_velocity`): divide px/s by
  the pixels that map to one unit of the property. Raw px/s on a 0–1 property overshoots
  wildly. **C/I**
- **Reversals overshoot once, even at ζ = 1**, because the preserved velocity carries the
  value on before it turns. Clamp the *output* of bounded properties at presentation
  (opacity and colour to 0–1, sizes to ≥ 0) and keep the spring's state unclamped so the
  next retarget stays continuous. **C**
- **Gestures**: follow the pointer 1:1, then on release retarget a `SpatialFast` spring to
  the snap point nearest the projected endpoint, inheriting the gesture's velocity
  ([WWDC18](https://developer.apple.com/videos/play/wwdc2018/803/),
  [libadwaita](https://github.com/GNOME/libadwaita/blob/main/src/adw-swipe-tracker.c)).
  Overshoot fits momentum; without it a flick "can often feel broken". **C**
- **Tweens restart** from the current value with zero velocity (`Tween::set_target`). That
  is fine for fades; it is why spatial motion uses springs.
- **Completion**: run "done" logic after the perceptual duration; release resources when
  the spring reports settled. **C**
- **View Transitions** suit discrete navigation, not live tiles: a new transition skips
  the running one to its end ([Chrome](https://developer.chrome.com/docs/web-platform/view-transitions/same-document)). **C**

## 8. Live data

| Rule | Tag |
|---|---|
| Routine updates never move a tile; they change text, colour or opacity | S (derived from [Abrams & Christ](https://pubmed.ncbi.nlm.nih.gov/12930472/)) |
| Spatial onsets only for state changes that warrant interruption: needs input, failed, finished | S/I |
| A re-sort animates all tiles together (`SpatialDefault`), never staggered | S ([Heer & Robertson](https://idl.cs.washington.edu/files/2007-AnimatedTransitions-InfoVis.pdf), [Chevalier et al.](https://hal.archives-ouvertes.fr/hal-01054408v1)) |
| Coalesce stream updates to at most one visual update per frame, and to about 1–4 per second for chips and counters | I |
| Retarget in-flight animations instead of queueing new ones | I |
| Counters: tabular numerals; roll only small deltas; snap when updates arrive faster than the roll | C/I ([NumberFlow](https://github.com/barvian/number-flow)) |
| Progress is monotonic and tweens 200–500 ms toward each new value; M3 uses `tween(500 ms, linear)` | C ([WavyProgressIndicator.kt](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/WavyProgressIndicator.kt)) |
| Progress stays indeterminate until the first real estimate, and never stalls near the end | C/I ([Harrison](https://www.chrisharrison.net/index.php/Research/ProgressBars)) |
| Progress waves and other ambient motion run only while progress is actually advancing, flatten at the ends, stop under reduced motion, when hidden, and on the global pause | S/C (2.2.2, M3) |
| Change highlights are colour-only, fade over about 1–2 s, and are rate-limited per element | I |
| Chart transitions use keyed joins and stage complex changes; cut or crossfade unrelated data | S/C ([Bostock](https://bost.ocks.org/mike/constancy/)) |

## 9. Reduced motion

CSS defines `reduce` as an interface that "removes or replaces" motion that causes
vestibular discomfort *or distraction for those with attention deficits*
([CSSWG](https://drafts.csswg.org/mediaqueries-5/#prefers-reduced-motion)), which brings
ambient tile motion into scope. Apple asks for "a dissolve, highlight fade, or color shift"
and warns that removing animation unnecessarily "might make your app experience more
confusing" ([Apple](https://developer.apple.com/help/app-store-connect/manage-app-accessibility/reduced-motion-evaluation-criteria)).
libadwaita makes the spring critically damped rather than removing it
([adw-bottom-sheet.c](https://github.com/GNOME/libadwaita/blob/main/src/adw-bottom-sheet.c)).

| Motion | Under reduce | Who decides | Tag |
|---|---|---|---|
| Gesture tracking | Keep 1:1; the release spring becomes ζ = 1 | `spring_for_role` | C ([HIG](https://developer.apple.com/design/human-interface-guidelines/accessibility)) |
| Small spatial (press, toggle, ≤ about 8 px) | Same tier at ζ = 1, or a colour/opacity cue only | `spring_for_role` | C/I |
| Medium spatial (≤ one tile) | Same tier at ζ = 1: still moving, no bounce | `spring_for_role` | C |
| Large travel, re-sort, tile → detail | Crossfade in place (`EffectsDefault`), or an instant move plus a 150–200 ms highlight fade | content layer, via `prefers_crossfade` | C |
| Scale or zoom, parallax, animated blur, depth | Opacity only, or off | content layer | C |
| Entrance cascade | One group fade, no stagger | content layer | I |
| Ambient loops, spinners, waves, pulses | Stop; show a static glyph or a determinate value | content layer | S/C |
| Effects (opacity, colour) | Keep, ≤ 200 ms | `spring_for_role` (unchanged) | C |
| Counters | Snap to the new value | content layer | C |

**What the crate does.** `MotionPolicy::spring_for_role(&scheme, role)` (used by
`MotionEngine::spring_for_role`):

| Policy | Spatial roles | Effects roles |
|---|---|---|
| `Full` | unchanged | unchanged |
| `Reduced` | same stiffness, ζ raised to 1 | unchanged |
| `Off` | snap | snap |

The older `MotionPolicy::spring_config(base)` has no role and keeps its old behaviour
(ζ = 1 at double stiffness for every spring under `Reduced`). Tweens under `Reduced` are
capped at 120 ms, linear.

**What the content layer does.** Substitution needs to know what is moving, so it belongs
to the layer that owns the content, such as a tile layer's motion code. It should:

- call `policy.prefers_crossfade(distance_px)` before animating a move, and crossfade when
  it returns true (re-sorts and zooms count as large travel regardless of distance);
- give every cascade zero delays when `policy != MotionPolicy::Full`;
- stop loops when `policy != MotionPolicy::Full`;
- snap counters when `policy != MotionPolicy::Full`.

## 10. Accessibility limits

These are binding. Themes cannot relax them.

| Limit | Rule | Tag |
|---|---|---|
| WCAG 2.2.2 Pause, Stop, Hide (A) | Auto-updating content shown alongside other content needs a way to pause, stop or hide it, or to control the update frequency. It has **no** 5-second exemption. Dashboards provide a global "pause live updates" control and a refresh-rate selector, and paused data shows "delayed, as of …". | S ([W3C](https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html)) |
| WCAG 2.2.2, moving content | Anything moving, blinking or scrolling for more than 5 s can be paused, stopped or hidden | S |
| In-app control | Honouring the OS setting alone is an incomplete mechanism, since most people never set it; provide a visible in-app control | I |
| WCAG 2.3.1 Three Flashes (A) | **At most 3 flashes in any one second, counted across all tiles that flash at once.** Exempt only if the combined flashing area is ≤ 0.006 sr (W3C: 341 × 256 px at 1024 × 768). A flash is an opposing luminance change of ≥ 10% where the darker state is below 0.80 relative luminance. | S ([W3C](https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html)) |
| Red | Saturated red never blinks | S/I |
| Patterns | Avoid high-contrast moving stripes | I |
| WCAG 2.3.3 Animation from Interactions (AAA) | Interaction-triggered motion can be disabled unless essential. Colour, blur and opacity changes are not "motion" under this criterion, which is why effects survive reduced motion. | S ([W3C](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html)) |
| Vestibular triggers | Large travel relative to the viewport, depth (zoom, parallax, 3D), animated blur, motion against the input direction, several speeds at once | C ([A List Apart](https://alistapart.com/article/designing-safer-web-animation-for-motion-sensitivity/), [Apple](https://developer.apple.com/help/app-store-connect/manage-app-accessibility/reduced-motion-evaluation-criteria)) |

About 35% of US adults aged 40 and over show vestibular dysfunction in balance testing
([NHANES via EurekAlert](https://www.eurekalert.org/news-releases/639509)). That measures
dysfunction, not sensitivity to screen motion, but the population is not small.

## 11. Performance

- **Compositor-only properties.** Animate `transform`, `opacity` and `filter`; Chromium
  accelerates only these by default ([Chrome](https://developer.chrome.com/blog/hardware-accelerated-animations)).
  Never animate width, height, top or left; use FLIP. **C**
- **Frame budget.** Your own work fits in about 10 ms of a 16.7 ms frame at 60 Hz, and
  proportionally less at 120 Hz (8.3 ms frames) ([web.dev](https://web.dev/articles/rendering-performance)). **C**
- **`will-change`** is a last resort, toggled from script around the animation
  ([MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/will-change)). **C**
- **Idle at rest.** Stop requesting frames when `MotionEngine::needs_frame()` is false. Zero
  CPU at rest, no infinite spinners ([WebKit](https://webkit.org/blog/8970/how-web-content-can-affect-power-usage/)). **C**
- **Pause when hidden**, offscreen or blurred: `Scheduler::pause()` / `resume()`, or a
  `TimeGapPolicy` for long gaps. Electron apps often disable background throttling, so the
  app must do this itself ([VS Code issue](https://www.github.com/microsoft/vscode/issues/22900)). **C**
- **Evaluate in closed form.** weaver-motion evaluates springs exactly for any frame
  interval: a 500 ms hitch jumps to the correct state, and stiff springs never blow up the
  way integrators do at 30–60 Hz ([Gaffer on Games](https://gafferongames.com/post/integration_basics/)). **C**
- **Sub-threshold tails** are what keep a surface awake. Per-unit thresholds (section 3)
  end them as soon as they are invisible. **I**

## 12. Testing motion deterministically

The engine has no clock of its own: every frame is `tick(timestamp)` with a timestamp you
choose, and the same timestamps give bit-identical output. Tests therefore run motion in
virtual time.

```rust
use weaver_motion::{MotionEngine, MotionPolicy, MotionRole, MotionScheme};

let mut engine = MotionEngine::with_policy(MotionPolicy::Reduced);
engine.set_scheme(MotionScheme::expressive());
let x = engine.spring_for_role("x", 0.0, MotionRole::SpatialFast);
engine.tick(0.0);
engine.retarget(x, 1.0);

let mut peak = 0.0_f64;
for frame in 1..=120 {
    engine.tick(frame as f64 / 60.0);
    peak = peak.max(engine.value_f64(x).unwrap());
}
assert!(peak <= 1.0);           // reduced motion never bounces
assert!(!engine.needs_frame()); // and goes idle
```

What to assert:

| Property | Assertion |
|---|---|
| Reduced motion | No overshoot on spatial roles; still part-way after one frame (moving, not snapped) |
| Idle at rest | `needs_frame()` is false within the role's 1e-3 settle time |
| Stagger | `stagger_delays(n)` spread ≤ 210 ms, at most 8 distinct delays |
| Flashes | Count opposing luminance transitions across all tiles in any 1 s window: ≤ 3 |
| Live data | A data-only update produces no change in any position or scale value |
| Retargeting | Value and velocity are unchanged by `retarget`; no jump on the next frame |
| Frame-rate independence | The same scenario at 30, 60 and 144 Hz agrees at shared timestamps |

On the web, Playwright's `page.clock` virtualises `requestAnimationFrame`, timers and
`performance` ([Playwright](https://playwright.dev/docs/clock)). Compose's `mainClock`
steps frame by frame for mid-animation goldens
([Android](https://developer.android.com/develop/ui/compose/animation/testing)). **C**

## 13. Converting between parameter styles

Every mainstream spring API is the same damped oscillator under different names. With
`k` stiffness, `m` mass (1 unless stated), `ζ` damping ratio, `c` damping coefficient,
`ω = √(k/m)`:

| Style | Parameters | To weaver-motion (k, ζ) | From weaver-motion |
|---|---|---|---|
| weaver-motion, Compose, Material 3 | stiffness, dampingRatio | k = stiffness, ζ = dampingRatio | — |
| Apple `Spring(duration:bounce:)`, Motion `bounce` | duration d (s), bounce b ∈ (−1, 1] | k = (2π/d)²; ζ = 1 − b (b ≥ 0), ζ = 1/(1 + b) (b < 0). `SpringConfig::from_duration_bounce(d, b)` | d = 2π/√k (`perceptual_duration()`); b = 1 − ζ (ζ ≤ 1), 1/ζ − 1 (ζ > 1) (`bounce()`) |
| SwiftUI `spring(response:dampingFraction:)` | response r (s), dampingFraction f | k = (2π/r)², ζ = f. `SpringConfig::from_response(r, f)` | r = 2π/√k, f = ζ |
| Apple damping coefficient | damping c | c = 4πζ/d = 2ζ√k | — |
| Motion (Framer Motion) physics | stiffness, damping, mass | k = stiffness, m = mass, ζ = damping / (2√(k·m)). `SpringConfig::from_damping_coefficient(mass, stiffness, damping)` | damping = 2ζ√(k·m) |
| Motion `visualDuration` + `bounce` | vd (s), b | d = 1.2·vd, then as Apple | vd = d / 1.2 |
| react-spring | tension, friction, mass | k = tension, ζ = friction / (2√(tension·mass)) | tension = k, friction = 2ζ√(k·m) |
| UIKit `usingSpringWithDamping:initialSpringVelocity:` | damping ratio, velocity normalised by distance | ζ = damping; velocity = v / total distance | — |
| CSS | `linear()` sampled from a spring | No velocity input: an interrupted CSS spring restarts at zero velocity | sample `solve` into stops |

Two traps:

- **The WWDC23 transcript formula "damping = 1 − 4π × bounce ÷ duration" is garbled.** At
  bounce 0 it gives ζ ≈ 0.04, an almost undamped spring. The correct form is
  `c = 4π(1 − bounce)/duration`. Apple's own example confirms it:
  `Spring(duration: 0.5, bounce: 0.3)` is stiffness 157.9, damping 17.6
  ([Apple Spring](https://developer.apple.com/documentation/swiftui/spring)).
- **Motion's `duration` + `bounce` (without `visualDuration`)** solves for ω numerically
  and drops inherited velocity so it ends exactly on time. It has no closed-form
  conversion. Also, negative bounce: weaver-motion follows the plain mapping, so
  bounce −0.3 at 0.5 s settles (2%) in about 798 ms against 464 ms at bounce 0; Motion
  rescales ω to keep arrival time.

Worked example, the standard `SpatialDefault` (k 700, ζ 0.9):

| Style | Values |
|---|---|
| Apple | duration 0.237 s, bounce 0.1 |
| SwiftUI response | response 0.237, dampingFraction 0.9 |
| Motion | visualDuration 0.198 s, bounce 0.1; or stiffness 700, damping 47.6, mass 1 |
| react-spring | tension 700, friction 47.6 |
| Compose | `spring(dampingRatio = 0.9f, stiffness = 700f)` |

Compose's named stiffnesses as Apple durations: High 10 000 → 63 ms, Medium 1500 → 162 ms,
MediumLow 400 → 314 ms, Low 200 → 444 ms, VeryLow 50 → 889 ms. Convert presets between
platforms through `d = 2π/√k`, never by copying stiffness numbers.

## 14. What themes may change

| A theme may | A theme may not |
|---|---|
| Quantise the output (for example snap to a 2 px grid), keeping the spring state continuous | Change which role a kind of change uses |
| Render effects as held steps, within 100–500 ms | Add bounce to effects, or bounce above the scheme's |
| Run its own art cadence (for example 12 fps) for decorative art | Loop without a pause, stop or hide, or break the 3-flashes-per-second limit |
| Pick offsets and distances within section 5 | Change the reduced-motion mapping, or the live-data rules |
| Tighten any budget (shorter stagger, fewer items) | Loosen any budget |

A quantised theme must keep its settle threshold at or below one grid step so a value
cannot oscillate between two cells. **I**

## 15. Sources

Perception and accessibility:
[Abrams & Christ 2003](https://pubmed.ncbi.nlm.nih.gov/12930472/) ·
[Isoluminant motion onset](https://neuropsychologylab.psych.utoronto.ca/files/Isoluminant%20motion%20onset%20captures%20attention.pdf) ·
[Maglio & Campbell 2000](https://www.interruptions.net/literature/Maglio-CHI00-p241-maglio.pdf) ·
[Heer & Robertson 2007](https://idl.cs.washington.edu/files/2007-AnimatedTransitions-InfoVis.pdf) ·
[Chevalier, Dragicevic & Franconeri 2014](https://hal.archives-ouvertes.fr/hal-01054408v1) ·
[Michotte launching](https://link.springer.com/content/pdf/10.3758%2FBF03194815.pdf) ·
[NN/g response times](https://www.nngroup.com/articles/response-times-3-important-limits/) ·
[NN/g animation duration](https://www.nngroup.com/articles/animation-duration/) ·
[NHANES vestibular](https://www.eurekalert.org/news-releases/639509) ·
[A List Apart](https://alistapart.com/article/designing-safer-web-animation-for-motion-sensitivity/) ·
[CSS Media Queries 5](https://drafts.csswg.org/mediaqueries-5/#prefers-reduced-motion) ·
[WCAG 2.2.2](https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html) ·
[WCAG 2.3.1](https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html) ·
[WCAG 2.3.3](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html)

Platforms:
[Apple WWDC23 Animate with springs](https://developer.apple.com/videos/play/wwdc2023/10158/) ·
[Apple WWDC18 Designing fluid interfaces](https://developer.apple.com/videos/play/wwdc2018/803/) ·
[Apple Spring](https://developer.apple.com/documentation/swiftui/spring) ·
[Apple reduced-motion criteria](https://developer.apple.com/help/app-store-connect/manage-app-accessibility/reduced-motion-evaluation-criteria) ·
[Apple HIG accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility) ·
[Material 3 standard tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/StandardMotionTokens.kt) ·
[MDC Motion](https://github.com/material-components/material-components-android/blob/master/docs/theming/Motion.md) ·
[Google expressive research](https://design.google/library/expressive-material-design-google-research) ·
[Compose visibility thresholds](https://github.com/androidx/androidx/blob/androidx-main/compose/animation/animation-core/src/commonMain/kotlin/androidx/compose/animation/core/VisibilityThresholds.kt) ·
[Carbon motion](https://carbondesignsystem.com/elements/motion/overview/) ·
[Carbon choreography](https://carbondesignsystem.com/elements/motion/choreography/) ·
[Fluent 2 motion](https://fluent2.microsoft.design/motion) ·
[Primer motion tokens](https://github.com/primer/primitives/blob/main/src/tokens/functional/motion/motion.json5) ·
[libadwaita swipe tracker](https://github.com/GNOME/libadwaita/blob/main/src/adw-swipe-tracker.c) ·
[Motion spring.ts](https://github.com/motiondivision/motion/blob/main/packages/motion-dom/src/animation/generators/spring.ts)

Engineering:
[Ryan Juckett, damped springs](https://www.ryanjuckett.com/damped-springs/) ·
[Gaffer on Games](https://gafferongames.com/post/integration_basics/) ·
[objc.io interactive animations](https://www.objc.io/issues/12-animations/interactive-animations/) ·
[FLIP](https://aerotwist.com/blog/flip-your-animations/) ·
[Chrome compositor animations](https://developer.chrome.com/blog/hardware-accelerated-animations) ·
[web.dev rendering performance](https://web.dev/articles/rendering-performance) ·
[WebKit power](https://webkit.org/blog/8970/how-web-content-can-affect-power-usage/) ·
[Playwright clock](https://playwright.dev/docs/clock) ·
[Compose animation testing](https://developer.android.com/develop/ui/compose/animation/testing) ·
[Bostock, object constancy](https://bost.ocks.org/mike/constancy/)
