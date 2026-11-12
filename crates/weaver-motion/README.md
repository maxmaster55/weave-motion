# Weave — portable motion engine

`weaver-motion` is the renderer-independent heart of the Weave engine: exact-time
damped springs, tweens, motion tokens and a shared scheduler.

```rust
use weaver_motion::{Spring, SpringConfig};

let mut spring = Spring::new(0.0, 1.0, SpringConfig::new(1.0, 300.0, 0.8)?);
spring.set_target(1.0); // keeps value *and* velocity: no restart from zero
spring.advance(1.0 / 60.0);
```

Choose springs by motion role (`MotionScheme`, `MotionRole`) rather than the
older `MotionToken` presets. The motion guide, `docs/MOTION.md` in the
repository, covers roles, durations, stagger, live data, reduced motion and
accessibility limits.

## Features

* `std` (default) — enable the standard library.
* no `std` — build for `no_std` targets; `alloc` is always required.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
