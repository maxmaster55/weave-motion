//! The DOM renderer.
//!
//! Only compiled for `wasm32` with the `web` feature: the rest of the
//! crate is platform independent.

use alloc::collections::BTreeMap;

use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement};

use weave_scene::{MotionRenderer, SceneSnapshot};

use crate::style::DomStyle;

/// Applies the presentation state of a scene to DOM elements.
///
/// The renderer keeps one DOM element per scene element. The application
/// registers the elements once and then only has to hand over snapshots.
///
/// The Rust API works with a whole [`SceneSnapshot`]; the `wasm_bindgen`
/// API below exposes the same renderer to JavaScript with plain numbers,
/// because a snapshot cannot cross the FFI boundary.
pub struct DomRenderer {
    #[allow(dead_code)]
    document: Document,
    elements: BTreeMap<u64, HtmlElement>,
    frame: u32,
}

impl MotionRenderer for DomRenderer {
    fn present(&mut self, snapshot: &SceneSnapshot) {
        DomRenderer::present(self, snapshot);
    }

    fn schedule_frame(&mut self) {
        // Frame scheduling is the host's responsibility.
    }
}

impl DomRenderer {
    /// Creates a renderer for the current document.
    pub fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;
        Ok(Self {
            document,
            elements: BTreeMap::new(),
            frame: 0,
        })
    }

    /// Registers the DOM element of a scene element.
    pub fn register(&mut self, id: u64, element: Element) {
        if let Ok(html) = element.clone().dyn_into::<HtmlElement>() {
            self.elements.insert(id, html);
        }
    }

    /// Removes the DOM element of a scene element.
    pub fn unregister(&mut self, id: u64) {
        self.elements.remove(&id);
    }

    /// Applies a snapshot to the registered elements.
    pub fn present(&mut self, snapshot: &SceneSnapshot) {
        for presentation in &snapshot.presentations {
            let id = presentation.id.raw();
            let Some(element) = self.elements.get(&id) else {
                continue;
            };
            let style = DomStyle::for_presentation(presentation);
            apply(element, &style);
        }
        self.frame = self.frame.wrapping_add(1);
    }

    /// The number of registered elements.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Whether no element is registered.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// The number of frames that were presented.
    pub fn frames(&self) -> u32 {
        self.frame
    }
}

/// The JavaScript facing surface of the renderer.
///
/// The snapshot cannot cross the FFI boundary, so JavaScript passes the
/// values of one element at a time.
#[wasm_bindgen]
pub struct DomRendererHandle {
    inner: DomRenderer,
}

#[wasm_bindgen]
impl DomRendererHandle {
    /// Creates a renderer for the current document.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<DomRendererHandle, JsValue> {
        Ok(DomRendererHandle {
            inner: DomRenderer::new()?,
        })
    }

    /// Registers the DOM element of a scene element.
    pub fn register(&mut self, id: u64, element: Element) {
        self.inner.register(id, element);
    }

    /// Removes the DOM element of a scene element.
    pub fn unregister(&mut self, id: u64) {
        self.inner.unregister(id);
    }

    /// Applies the state of one element.
    pub fn apply(
        &mut self,
        id: u64,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        opacity: f32,
        top_left_radius: f32,
        top_right_radius: f32,
        bottom_right_radius: f32,
        bottom_left_radius: f32,
    ) {
        let style = DomStyle {
            left: format!("{left}px"),
            top: format!("{top}px"),
            width: format!("{}px", width.max(0.0)),
            height: format!("{}px", height.max(0.0)),
            opacity: format!("{}", opacity.clamp(0.0, 1.0)),
            clip_path: Some(format!(
                "inset(0px 0px 0px 0px round {}px {}px {}px {}px)",
                top_left_radius, top_right_radius, bottom_right_radius, bottom_left_radius
            )),
            transform: None,
        };

        if let Some(element) = self.inner.elements.get(&id) {
            apply(element, &style);
        }
    }

    /// The number of registered elements.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether no element is registered.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

fn apply(element: &HtmlElement, style: &DomStyle) {
    let inline = element.style();
    let _ = inline.set_property("left", &style.left);
    let _ = inline.set_property("top", &style.top);
    let _ = inline.set_property("width", &style.width);
    let _ = inline.set_property("height", &style.height);
    let _ = inline.set_property("opacity", &style.opacity);

    if let Some(clip_path) = &style.clip_path {
        let _ = inline.set_property("clip-path", clip_path);
    } else {
        let _ = inline.remove_property("clip-path");
    }

    if let Some(transform) = &style.transform {
        let _ = inline.set_property("transform", transform);
    } else {
        let _ = inline.remove_property("transform");
    }
}
