---
name: weave-motion-engine
summary: Implement or integrate the Weave renderer-independent Rust spring animation and shape morphing engine.
---

# Skill: Weave Motion Engine

Use when the user asks to implement, extend, optimize, port, or apply **Weave**, the expressive UI motion system intended for projects such as Weaver and Loom.

## Essential reading

1. Read [DESIGN.md](DESIGN.md) before choosing an architecture.
2. Read [references/PHYSICS.md](references/PHYSICS.md) for the exact-time spring model and scheduler behavior.
3. Read [references/MORPHING.md](references/MORPHING.md) before implementing geometry.
4. Read [references/INTEGRATION.md](references/INTEGRATION.md) for stable identity, layout, renderer and policy.
5. Read [references/TESTING.md](references/TESTING.md) for minimum tests.

## Agent workflow

1. Inspect the actual repository, dependencies, rendering stack, input and layout APIs; state findings and integration boundaries. Do not fabricate existing modules.
2. Design the minimal standalone `weave-core` crate. Implement spring state, target retargeting, exact-time update, settling, tokens and scheduling with tests.
3. Build a runnable demonstration with interactive retargeting before adding complicated geometry.
4. Implement parametric rounded-rectangle morphs and a pill-to-panel demo. Move to compatible Bézier path morphs only after stable output.
5. Add shared IDs, FLIP bounds transitions, choreography and content reveal as separate concerns.
6. Add the narrowest necessary renderer adapter; preserve a renderer-independent core.
7. Verify reduced-motion behavior, interruption, idle-frame behavior and frame budgets.
8. Report source changes, test commands/results, performance caveats, and unfinished work.

## Non-negotiable technical rules

- Do not restart velocities on interruption, unless a specific documented animation intentionally opts in.
- Do not swap a moving panel with a disconnected fade when coherent shape and layout transforms are possible.
- Do not assume different shapes have aligned contour point order.
- Do not stretch readable text as if it were a vector path.
- Do not allow morph overshoot to invalidate geometry.
- Do not couple spring solver to wgpu, UI frameworks or application-specific objects.
- Do not use arbitrary timers per widget or force perpetual redraws.
- Keep animation meaningful, avoid gratuitous bounces, and respect reduced motion.
- Treat example APIs, parameters and preset names as proposals, not existing stable public contracts.

## First deliverable

A working Rust workspace with a unit-tested core and a demo where a pill-shaped button expands into a content panel, can reverse at any point, and settles without snapping or excess redraws. Keep code modular enough that a second application can reuse it unchanged.
