# Prompt to give a coding agent

Build **Weave**, a reusable Rust animation engine inspired by Material 3 Expressive, using the attached `SKILL.md`, `DESIGN.md` and references as the authoritative project specification.

It must be renderer-independent at its core. Prioritize velocity-preserving, retargetable spring animations, an exact-time damped-spring solver, a common scheduler, parametric shape morphing, stable element identity and the ability to animate button-to-panel transitions without visual jumps.

Start by inspecting my repository and explaining what already exists and what can be reused. Do not invent APIs or files. Implement incrementally in working, testable Rust crates. Provide a spring playground and then a pill-to-panel demo. The animation must be interruptible and reversible mid-flight, remain accessible with reduced motion, and stop scheduling frames when idle.

Only then extend to Bézier contour morphing, shared-element FLIP choreography, optional wgpu rendering, and other adapters. Separate animation state, target layout, and presentation geometry. Preserve readable content; avoid applying shape deformation to text. Do not claim full arbitrary topology morphs in the first release.

For each milestone, run unit tests and available demo builds, document design decisions and output actual results. Favor clear code and a narrow correct MVP over speculative framework features.
