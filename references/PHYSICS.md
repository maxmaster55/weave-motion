# Spring physics and scheduling

## Continuous-time model

`m x'' + c x' + k(x - target) = 0`, with `omega = sqrt(k/m)` and `zeta = c/(2 sqrt(km))`.

Let `y = x - target` and initial values `y0`, `v0`. For a constant target during `dt`:

**Underdamped (`zeta < 1`)**

- `alpha = zeta * omega`, `wd = omega * sqrt(1-zeta²)`
- `A = y0`, `B = (v0 + alpha*y0) / wd`
- `y(t) = exp(-alpha*t) [A cos(wd*t) + B sin(wd*t)]`
- `v(t) = exp(-alpha*t) [(B*wd-alpha*A) cos(wd*t) + (-A*wd-alpha*B) sin(wd*t)]`

**Critically damped (`zeta = 1`)**

- `B = v0 + omega*y0`
- `y(t) = (y0 + B*t) exp(-omega*t)`
- `v(t) = (B - omega*(y0+B*t)) exp(-omega*t)`

**Overdamped (`zeta > 1`)**

- `r1 = -omega * (zeta - sqrt(zeta²-1))`, `r2 = -omega * (zeta + sqrt(zeta²-1))`
- `C1 = (v0-r2*y0)/(r1-r2)`, `C2 = y0-C1`
- `y(t) = C1 exp(r1*t) + C2 exp(r2*t)`
- `v(t) = r1*C1 exp(r1*t) + r2*C2 exp(r2*t)`

Return `(target+y(t), v(t))`. Use numerically stable branching near critical damping, limit divisions by tiny `wd` and test convergence. Validate mass/stiffness > 0, damping ratio >= 0, inputs finite.

**Target change:** integrate from previous instant to event timestamp, set new target, keep `(x,v)` unchanged, then integrate forward. Do not reinitialize a keyframe or reset velocity.

**Animation endpoints:** settle with both `abs(x-target)<position_epsilon` AND `abs(v)<velocity_epsilon`. Thresholds depend on property units (pixels, radians, scalar). Snap to target and zero velocity *only once settled* to avoid micro-updates.

## Scheduler contract

- Single monotonic clock drives active animations; accept timestamp or elapsed seconds, never wall clock.
- Use timestamps from rendering framework when possible.
- No continuous wakeups while all springs are settled.
- When UI target changes, schedule next frame immediately.
- For large frame gaps, document behavior: analytic fast-forward (default), pause/freeze in background, or snap after suspension.
- Keep physics updates independent from vsync cadence; visual presentation draws latest snapshot.
- Include pause/resume, cancellation, removal and listener lifecycle.

## Optional fallback integrator

Semi-implicit Euler per substep: `a=(-k*(x-target)-c*v)/m; v += a*h; x += v*h`. Use at most a bounded number of stable substeps and consider exact-form update as production default. Substepping must not silently introduce a different notion of elapsed time after stalls.

## Presets (invented examples, not Google tokens)

| Name | Mass | Stiffness | Zeta | Motion |
| --- | ---: | ---: | ---: | --- |
| fast_spatial | 1 | 500 | 0.85 | quick feedback |
| default_spatial | 1 | 300 | 0.80 | everyday movement |
| expressive_spatial | 1 | 220 | 0.65 | hero transitions |
| gentle_spatial | 1 | 140 | 1.00 | large surfaces |

Effects such as alpha/color are often better as short tweens (e.g. 150–200ms initial tuning range) with intentionally chosen easing. Avoid uncontrolled transparency overshoot; clamp or use effects-specific interpolation.
