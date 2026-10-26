//! The scene: stable identity, layout destinations and presentation state.
//!
//! The scene is the bridge between an application's layout and Weave's
//! motion:
//!
//! 1. the application declares **destinations** (where an element should
//!    be, which shape it should have),
//! 2. the scene keeps the **presentation** (where it currently is) and
//!    animates the difference,
//! 3. the scene hands the presentation to the renderer.
//!
//! The scene never owns layout or input, and it never restarts a motion
//! from zero: interrupting a transition keeps the current value *and* the
//! current velocity.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use weave_core::{
    Animated, AnimationId, MotionEngine, MotionPolicy, MotionSnapshot, MotionToken, SpringConfig,
};
use weave_geometry::{Morph, Rect, Shape};

use crate::choreography::{Choreographer, Cue, Gate};
use crate::id::{ElementId, ElementRegistry};
use crate::transform::{Flip, Transform, TransitionMode};

/// The animated state of one element, ready to be rendered.
#[derive(Clone, Debug, PartialEq)]
pub struct Presentation {
    /// The stable identity of the element.
    pub id: ElementId,
    /// The key the element was registered under.
    pub key: String,
    /// The animated bounds of the element.
    pub bounds: Rect,
    /// The FLIP transform, when the element moves in flip mode.
    pub transform: Transform,
    /// The animated clipping shape, when the element has one.
    pub clip: Option<Shape>,
    /// The animated opacity, between 0 and 1.
    pub opacity: f64,
    /// The progress of the shape morph, between 0 and 1.
    pub morph_progress: f64,
    /// Whether the content of the element should be visible.
    pub content_visible: bool,
    /// How the element moves to its destination.
    pub mode: TransitionMode,
}

/// A snapshot of the whole scene.
#[derive(Clone, Debug, Default)]
pub struct SceneSnapshot {
    /// The timestamp the snapshot was produced for.
    pub timestamp: f64,
    /// Seconds elapsed since the previous frame.
    pub dt: f64,
    /// One presentation per element, in registration order.
    pub presentations: Vec<Presentation>,
    /// Whether another frame is needed.
    pub needs_frame: bool,
}

impl SceneSnapshot {
    /// Looks up the presentation of an element.
    pub fn get(&self, id: ElementId) -> Option<&Presentation> {
        self.presentations.iter().find(|item| item.id == id)
    }

    /// Looks up the presentation of an element by key.
    pub fn get_by_key(&self, key: &str) -> Option<&Presentation> {
        self.presentations.iter().find(|item| item.key == key)
    }
}

#[derive(Debug)]
struct ElementState {
    id: ElementId,
    key: String,
    destination: Rect,
    bounds: AnimationId,
    transform: AnimationId,
    morph: Option<AnimationId>,
    opacity: AnimationId,
    shape_from: Option<Shape>,
    shape_to: Option<Shape>,
    mode: TransitionMode,
    content_visible: bool,
    /// Whether the element was placed at least once.
    placed: bool,
}

/// A collection of elements with stable identity and animated presentation.
#[derive(Debug, Default)]
pub struct Scene {
    engine: MotionEngine,
    registry: ElementRegistry,
    elements: Vec<ElementState>,
    index: BTreeMap<u64, usize>,
    choreographer: Choreographer,
}

impl Scene {
    /// Creates an empty scene.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a scene with an explicit motion policy.
    pub fn with_policy(policy: MotionPolicy) -> Self {
        Self {
            engine: MotionEngine::with_policy(policy),
            ..Self::default()
        }
    }

    /// The active motion policy.
    pub fn policy(&self) -> MotionPolicy {
        self.engine.policy()
    }

    /// Changes the motion policy.
    pub fn set_policy(&mut self, policy: MotionPolicy) {
        self.engine.set_policy(policy);
    }

    /// The motion engine behind the scene.
    pub fn engine(&self) -> &MotionEngine {
        &self.engine
    }

    /// Mutable access to the motion engine.
    pub fn engine_mut(&mut self) -> &mut MotionEngine {
        &mut self.engine
    }

    /// Interns a stable key and returns its identifier.
    pub fn element(&mut self, key: &str) -> ElementId {
        if let Some(id) = self.registry.get(key) {
            return id;
        }

        let id = self.registry.intern(key);
        let config = self.default_spring();

        let bounds = self.insert_bounds(key, Rect::default(), config);
        let transform = self.insert_transform(key, Transform::IDENTITY, config);
        let opacity = self
            .engine
            .tween_resting(opacity_key(key), 1.0, MotionToken::Effects);

        self.index.insert(id.raw(), self.elements.len());
        self.elements.push(ElementState {
            id,
            key: key.to_string(),
            destination: Rect::default(),
            bounds,
            transform,
            morph: None,
            opacity,
            shape_from: None,
            shape_to: None,
            mode: TransitionMode::Bounds,
            content_visible: true,
            placed: false,
        });

        id
    }

    /// The identifier of an element, if it exists.
    pub fn element_id(&self, key: &str) -> Option<ElementId> {
        self.registry.get(key)
    }

    /// Declares the destination bounds of an element.
    ///
    /// The first call places the element without animating it. Later calls
    /// animate the movement from the current presentation to the new
    /// destination, keeping the velocity of an interrupted motion.
    pub fn layout(&mut self, key: &str, bounds: Rect) -> ElementId {
        self.layout_with_mode(key, bounds, TransitionMode::Bounds)
    }

    /// Declares the destination bounds of an element with an explicit
    /// transition mode.
    pub fn layout_with_mode(&mut self, key: &str, bounds: Rect, mode: TransitionMode) -> ElementId {
        let id = self.element(key);
        let position = self.position(id);
        let placed = self.elements[position].placed;
        let previous = self.elements[position].destination;

        self.elements[position].destination = bounds;
        self.elements[position].mode = mode;
        self.elements[position].placed = true;

        match mode {
            TransitionMode::Bounds => {
                if placed {
                    self.retarget_bounds(id, bounds);
                } else {
                    // The first placement is not an animation: the element
                    // simply appears where it belongs.
                    self.snap_bounds(id, bounds);
                }
            }
            TransitionMode::Flip => {
                let first = self.presentation_bounds(id);
                let flip = Flip::new(first, bounds);
                let invert = flip.invert();

                self.snap_bounds(id, bounds);
                if placed {
                    self.set_transform(id, invert, Transform::IDENTITY);
                } else {
                    self.set_transform(id, Transform::IDENTITY, Transform::IDENTITY);
                }
            }
        }

        // A new transition re-arms the choreography.
        if placed && previous != bounds {
            self.choreographer.rearm();
        }

        id
    }

    /// Declares the destination shape of an element and starts a morph.
    ///
    /// The morph starts from the shape the element currently shows, so an
    /// interrupted morph continues from the visible geometry instead of
    /// jumping back to the previous shape.
    pub fn shape(&mut self, key: &str, shape: Shape) -> ElementId {
        let id = self.element(key);
        let position = self.position(id);

        // The shape the element currently shows.
        let (progress, from, to) = {
            let state = &self.elements[position];
            let progress = state
                .morph
                .and_then(|morph| self.engine.value(morph))
                .and_then(|value| value.as_scalar())
                .unwrap_or(1.0);
            (progress, state.shape_from, state.shape_to)
        };

        let current = match (from, to) {
            (Some(from), Some(to)) => Morph::new(from, to).and_then(|morph| morph.apply(progress)),
            (_, Some(to)) => Some(to),
            _ => None,
        };

        let config = self.default_spring();
        let morph = self.engine.scheduler_mut().insert(
            Some(morph_key(key)),
            Box::new(Animated::new(0.0_f64, 1.0, config)),
        );

        let position = self.position(id);
        self.elements[position].shape_from = Some(current.unwrap_or(shape));
        self.elements[position].shape_to = Some(shape);
        self.elements[position].morph = Some(morph);

        id
    }

    /// Sets the opacity target of an element.
    pub fn opacity(&mut self, key: &str, opacity: f64) -> ElementId {
        let id = self.element(key);
        let position = self.position(id);
        let animation = self.elements[position].opacity;
        self.engine.retarget(animation, opacity);
        id
    }

    /// Shows or hides the content of an element without animating it.
    ///
    /// The content flag is what the renderer uses to decide whether to draw
    /// text and icons; the opacity is animated separately.
    pub fn set_content_visible(&mut self, key: &str, visible: bool) -> ElementId {
        let id = self.element(key);
        let position = self.position(id);
        self.elements[position].content_visible = visible;
        id
    }

    /// Sets how an element moves to its destination.
    pub fn transition_mode(&mut self, key: &str, mode: TransitionMode) {
        let id = self.element(key);
        let position = self.position(id);
        self.elements[position].mode = mode;
    }

    /// Registers a choreography dependency.
    pub fn cue(&mut self, gate: Gate, cue: Cue) {
        self.choreographer.place(gate, cue);
    }

    /// Removes every choreography dependency.
    pub fn clear_cues(&mut self) {
        self.choreographer.clear();
    }

    /// Advances the scene and returns a snapshot.
    pub fn tick(&mut self, timestamp: f64) -> SceneSnapshot {
        let MotionSnapshot { dt, .. } = self.engine.tick(timestamp);

        self.evaluate_choreography();

        let presentations = self
            .elements
            .iter()
            .map(|state| self.presentation_of(state))
            .collect();

        SceneSnapshot {
            timestamp,
            dt,
            presentations,
            needs_frame: self.engine.needs_frame(),
        }
    }

    /// The current presentation of an element.
    pub fn presentation(&self, key: &str) -> Option<Presentation> {
        let id = self.registry.get(key)?;
        let position = *self.index.get(&id.raw())?;
        Some(self.presentation_of(&self.elements[position]))
    }

    /// `true` when nothing is moving any more.
    pub fn is_idle(&self) -> bool {
        self.engine.is_idle()
    }

    /// `true` when at least one element is still moving.
    pub fn needs_frame(&self) -> bool {
        self.engine.needs_frame()
    }

    /// The number of elements in the scene.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Whether the scene is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    fn evaluate_choreography(&mut self) {
        let ready = self.choreographer.ready(|gate| self.gate_satisfied(gate));

        for index in ready {
            let Some(cue) = self.choreographer.cue(index) else {
                continue;
            };
            self.choreographer.mark_fired(index);
            self.execute(cue);
        }
    }

    fn execute(&mut self, cue: Cue) {
        match cue {
            Cue::RevealContent { element } => {
                if let Some(position) = self.position_of(element) {
                    self.elements[position].content_visible = true;
                    let animation = self.elements[position].opacity;
                    self.engine.retarget(animation, 1.0);
                }
            }
            Cue::HideContent { element } => {
                if let Some(position) = self.position_of(element) {
                    self.elements[position].content_visible = false;
                    let animation = self.elements[position].opacity;
                    self.engine.retarget(animation, 0.0);
                }
            }
            Cue::SetOpacity { element, value } => {
                if let Some(position) = self.position_of(element) {
                    let animation = self.elements[position].opacity;
                    self.engine.retarget(animation, value);
                }
            }
        }
    }

    fn gate_satisfied(&self, gate: &Gate) -> bool {
        match gate {
            Gate::MorphProgress { element, threshold } => self
                .presentation_of_element(*element)
                .map(|presentation| presentation.morph_progress >= *threshold)
                .unwrap_or(false),
            Gate::WidthRatio { element, threshold } => {
                let Some(presentation) = self.presentation_of_element(*element) else {
                    return false;
                };
                let Some(position) = self.position_of(*element) else {
                    return false;
                };
                let target = self.elements[position].destination.width;
                if target <= 0.0 {
                    return false;
                }
                presentation.bounds.width / target >= *threshold
            }
            Gate::HeightRatio { element, threshold } => {
                let Some(presentation) = self.presentation_of_element(*element) else {
                    return false;
                };
                let Some(position) = self.position_of(*element) else {
                    return false;
                };
                let target = self.elements[position].destination.height;
                if target <= 0.0 {
                    return false;
                }
                presentation.bounds.height / target >= *threshold
            }
            Gate::Opacity { element, threshold } => self
                .presentation_of_element(*element)
                .map(|presentation| presentation.opacity >= *threshold)
                .unwrap_or(false),
        }
    }

    fn presentation_of(&self, state: &ElementState) -> Presentation {
        let bounds = self.animated_bounds(state.bounds);
        let transform = self.animated_transform(state.transform);
        let opacity = self
            .engine
            .value(state.opacity)
            .and_then(|value| value.as_scalar())
            .unwrap_or(1.0);
        let morph_progress = self.morph_progress(state);
        let clip = state.shape_from.zip(state.shape_to).and_then(|(from, to)| {
            let morph = Morph::new(from, to)?;
            morph.apply(morph_progress)
        });

        Presentation {
            id: state.id,
            key: state.key.clone(),
            bounds,
            transform,
            clip,
            opacity,
            morph_progress,
            content_visible: state.content_visible,
            mode: state.mode,
        }
    }

    fn presentation_of_element(&self, id: ElementId) -> Option<Presentation> {
        let position = self.position_of(id)?;
        Some(self.presentation_of(&self.elements[position]))
    }

    fn morph_progress(&self, state: &ElementState) -> f64 {
        state
            .morph
            .and_then(|morph| self.engine.value(morph))
            .and_then(|value| value.as_scalar())
            .unwrap_or(if state.shape_to.is_some() { 1.0 } else { 0.0 })
    }

    fn animated_bounds(&self, id: AnimationId) -> Rect {
        self.engine
            .scheduler()
            .get_animated::<Rect>(id)
            .map(|animated| animated.value())
            .unwrap_or_default()
    }

    fn animated_transform(&self, id: AnimationId) -> Transform {
        self.engine
            .scheduler()
            .get_animated::<Transform>(id)
            .map(|animated| animated.value())
            .unwrap_or_default()
    }

    fn presentation_bounds(&self, id: ElementId) -> Rect {
        let Some(position) = self.position_of(id) else {
            return Rect::default();
        };
        self.animated_bounds(self.elements[position].bounds)
    }

    fn insert_bounds(&mut self, key: &str, initial: Rect, config: SpringConfig) -> AnimationId {
        self.engine.scheduler_mut().insert(
            Some(bounds_key(key)),
            Box::new(Animated::new(initial, initial, config)),
        )
    }

    fn insert_transform(
        &mut self,
        key: &str,
        initial: Transform,
        config: SpringConfig,
    ) -> AnimationId {
        self.engine.scheduler_mut().insert(
            Some(transform_key(key)),
            Box::new(Animated::new(initial, initial, config)),
        )
    }

    fn retarget_bounds(&mut self, id: ElementId, bounds: Rect) {
        let Some(position) = self.position_of(id) else {
            return;
        };
        let animation = self.elements[position].bounds;
        if let Some(value) = self
            .engine
            .scheduler_mut()
            .get_animated_mut::<Rect>(animation)
        {
            value.set_target(bounds);
        }
    }

    fn snap_bounds(&mut self, id: ElementId, bounds: Rect) {
        let Some(position) = self.position_of(id) else {
            return;
        };
        let animation = self.elements[position].bounds;
        if let Some(value) = self
            .engine
            .scheduler_mut()
            .get_animated_mut::<Rect>(animation)
        {
            value.snap(bounds);
        }
    }

    fn set_transform(&mut self, id: ElementId, from: Transform, to: Transform) {
        let Some(position) = self.position_of(id) else {
            return;
        };
        let animation = self.elements[position].transform;
        if let Some(value) = self
            .engine
            .scheduler_mut()
            .get_animated_mut::<Transform>(animation)
        {
            value.snap(from);
            value.set_target(to);
        }
    }

    fn default_spring(&self) -> SpringConfig {
        self.engine.policy().spring_config(
            MotionToken::DefaultSpatial
                .spring_config()
                .expect("spatial token"),
        )
    }

    fn position(&self, id: ElementId) -> usize {
        self.position_of(id).expect("the element exists")
    }

    fn position_of(&self, id: ElementId) -> Option<usize> {
        self.index.get(&id.raw()).copied()
    }
}

fn bounds_key(key: &str) -> String {
    let mut out = String::from("element:");
    out.push_str(key);
    out.push_str(":bounds");
    out
}

fn transform_key(key: &str) -> String {
    let mut out = String::from("element:");
    out.push_str(key);
    out.push_str(":transform");
    out
}

fn morph_key(key: &str) -> String {
    let mut out = String::from("element:");
    out.push_str(key);
    out.push_str(":morph");
    out
}

fn opacity_key(key: &str) -> String {
    let mut out = String::from("element:");
    out.push_str(key);
    out.push_str(":opacity");
    out
}
