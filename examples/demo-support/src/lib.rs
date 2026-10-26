//! Shared terminal helpers for the Weave demos.
//!
//! The demos deliberately avoid any windowing or GPU dependency: they must
//! run on a build machine, in CI and over SSH. Everything here is plain
//! ANSI escape sequences and a character grid.
//!
//! This crate is **not** published — it only exists to keep the demos short.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use std::io::IsTerminal;
use std::time::Instant;

/// ANSI escape sequences used by the demos.
pub mod ansi {
    /// Clears the screen and moves the cursor home.
    pub const CLEAR: &str = "\x1b[2J\x1b[H";
    /// Hides the cursor.
    pub const HIDE_CURSOR: &str = "\x1b[?25l";
    /// Shows the cursor.
    pub const SHOW_CURSOR: &str = "\x1b[?25h";
    /// Moves the cursor to the top left corner.
    pub const HOME: &str = "\x1b[H";
    /// Bold text.
    pub const BOLD: &str = "\x1b[1m";
    /// Dim text.
    pub const DIM: &str = "\x1b[2m";
    /// Resets all attributes.
    pub const RESET: &str = "\x1b[0m";
    /// Cyan foreground.
    pub const CYAN: &str = "\x1b[36m";
    /// Green foreground.
    pub const GREEN: &str = "\x1b[32m";
    /// Yellow foreground.
    pub const YELLOW: &str = "\x1b[33m";
    /// Magenta foreground.
    pub const MAGENTA: &str = "\x1b[35m";
}

/// `true` when standard output is attached to a terminal.
pub fn is_terminal() -> bool {
    std::io::stdout().is_terminal()
}

/// A monotonic clock for the demos, in seconds.
pub struct DemoClock {
    start: Instant,
}

impl DemoClock {
    /// Creates a clock starting now.
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Seconds since the clock was created.
    pub fn elapsed(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

impl Default for DemoClock {
    fn default() -> Self {
        Self::new()
    }
}

/// A character grid that can be rendered to the terminal.
#[derive(Clone)]
pub struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<char>,
    clip: Option<ClipRect>,
}

/// A rectangular clipping region, in canvas coordinates.
#[derive(Clone, Copy, Debug)]
pub struct ClipRect {
    /// Left edge.
    pub x: isize,
    /// Top edge.
    pub y: isize,
    /// Width of the region.
    pub width: usize,
    /// Height of the region.
    pub height: usize,
}

impl ClipRect {
    /// Whether a point lies inside the region.
    pub fn contains(&self, x: isize, y: isize) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x + self.width as isize
            && y < self.y + self.height as isize
    }
}

impl Canvas {
    /// Creates an empty canvas.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![' '; width * height],
            clip: None,
        }
    }

    /// The canvas width, in characters.
    pub fn width(&self) -> usize {
        self.width
    }

    /// The canvas height, in characters.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Restricts drawing to a rectangle. `None` removes the restriction.
    ///
    /// This is how the demos show that content is clipped by the shape of
    /// the surface it belongs to.
    pub fn set_clip(&mut self, clip: Option<ClipRect>) {
        self.clip = clip;
    }

    /// The active clipping region.
    pub fn clip(&self) -> Option<ClipRect> {
        self.clip
    }

    /// Clears the canvas.
    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|cell| *cell = ' ');
    }

    /// Writes a character, ignoring out-of-range coordinates and anything
    /// outside the clipping region.
    pub fn set(&mut self, x: isize, y: isize, ch: char) {
        if x < 0 || y < 0 {
            return;
        }
        if let Some(clip) = self.clip {
            if !clip.contains(x, y) {
                return;
            }
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            return;
        }
        self.cells[y * self.width + x] = ch;
    }

    /// Writes a string, ignoring out-of-range coordinates.
    pub fn text(&mut self, x: isize, y: isize, text: &str) {
        for (offset, ch) in text.chars().enumerate() {
            self.set(x + offset as isize, y, ch);
        }
    }

    /// Draws a horizontal line.
    pub fn hline(&mut self, y: isize, from: isize, to: isize, ch: char) {
        for x in from..=to {
            self.set(x, y, ch);
        }
    }

    /// Draws a vertical line.
    pub fn vline(&mut self, x: isize, from: isize, to: isize, ch: char) {
        for y in from..=to {
            self.set(x, y, ch);
        }
    }

    /// Draws the outline of a rounded rectangle.
    ///
    /// A `radius` of `0` produces a plain rectangle; a radius larger than
    /// half the shorter side produces a pill.
    pub fn rounded_rect(
        &mut self,
        x: isize,
        y: isize,
        width: usize,
        height: usize,
        radius: usize,
        ch: char,
    ) {
        self.rounded_rect_outline(x, y, width, height, radius, 1, ch);
    }

    /// Draws the outline of a rounded rectangle with a given thickness.
    ///
    /// Only the ring of the shape is drawn, so the interior stays readable
    /// — exactly what a UI surface needs.
    #[allow(clippy::too_many_arguments)]
    pub fn rounded_rect_outline(
        &mut self,
        x: isize,
        y: isize,
        width: usize,
        height: usize,
        radius: usize,
        thickness: usize,
        ch: char,
    ) {
        if width == 0 || height == 0 {
            return;
        }

        let max_radius = width.min(height) / 2;
        let r = radius.min(max_radius) as isize;
        let thickness = thickness.max(1) as isize;
        let w = width as isize;
        let h = height as isize;
        let last_x = x + w - 1;
        let last_y = y + h - 1;

        for row in 0..h {
            for column in 0..w {
                let px = x + column;
                let py = y + row;

                let in_corner = (column < r && row < r)
                    || (column > w - r - 1 && row < r)
                    || (column < r && row > h - r - 1)
                    || (column > w - r - 1 && row > h - r - 1);

                if r == 0 {
                    if px == x || px == last_x || py == y || py == last_y {
                        self.set(px, py, ch);
                    }
                    continue;
                }

                if in_corner {
                    let (cx, cy) = if column < r && row < r {
                        (x + r - 1, y + r - 1)
                    } else if column > w - r - 1 && row < r {
                        (last_x - r + 1, y + r - 1)
                    } else if column < r {
                        (x + r - 1, last_y - r + 1)
                    } else {
                        (last_x - r + 1, last_y - r + 1)
                    };

                    let dx = px - cx;
                    let dy = py - cy;
                    let distance = dx * dx + dy * dy;
                    let outer = r * r;
                    let inner = (r - thickness).max(0) * (r - thickness).max(0);

                    if distance <= outer && distance > inner {
                        self.set(px, py, ch);
                    }
                } else if px == x || px == last_x || py == y || py == last_y {
                    self.set(px, py, ch);
                }
            }
        }
    }

    /// Fills a rounded rectangle with a character.
    pub fn rounded_rect_fill(
        &mut self,
        x: isize,
        y: isize,
        width: usize,
        height: usize,
        radius: usize,
        ch: char,
    ) {
        if width == 0 || height == 0 {
            return;
        }

        let max_radius = width.min(height) / 2;
        let r = radius.min(max_radius) as isize;
        let w = width as isize;
        let h = height as isize;
        let last_x = x + w - 1;
        let last_y = y + h - 1;

        for row in 0..h {
            for column in 0..w {
                let px = x + column;
                let py = y + row;

                if r == 0 {
                    self.set(px, py, ch);
                    continue;
                }

                let in_corner = (column < r && row < r)
                    || (column > w - r - 1 && row < r)
                    || (column < r && row > h - r - 1)
                    || (column > w - r - 1 && row > h - r - 1);

                if in_corner {
                    let (cx, cy) = if column < r && row < r {
                        (x + r - 1, y + r - 1)
                    } else if column > w - r - 1 && row < r {
                        (last_x - r + 1, y + r - 1)
                    } else if column < r {
                        (x + r - 1, last_y - r + 1)
                    } else {
                        (last_x - r + 1, last_y - r + 1)
                    };

                    let dx = px - cx;
                    let dy = py - cy;
                    if dx * dx + dy * dy <= r * r {
                        self.set(px, py, ch);
                    }
                } else {
                    self.set(px, py, ch);
                }
            }
        }
    }

    /// Renders the canvas as a string, one line per row.
    pub fn render(&self) -> String {
        let mut out = String::with_capacity(self.width * self.height + self.height);
        for y in 0..self.height {
            if y > 0 {
                out.push('\n');
            }
            for x in 0..self.width {
                out.push(self.cells[y * self.width + x]);
            }
        }
        out
    }
}

/// Draws a sparkline-style plot of `values` into `canvas`.
///
/// The plot is normalised between `minimum` and `maximum` and drawn with
/// `*` characters; the target line is drawn with `-`.
pub fn plot(
    canvas: &mut Canvas,
    values: &[f64],
    minimum: f64,
    maximum: f64,
    origin: (isize, isize),
    size: (isize, isize),
) {
    let (ox, oy) = origin;
    let (width, height) = size;
    let span = maximum - minimum;
    if span <= 0.0 || values.is_empty() {
        return;
    }

    let plot_width = width as usize;
    for (index, value) in values.iter().enumerate() {
        if index >= plot_width {
            break;
        }
        let normalized = ((value - minimum) / span).clamp(0.0, 1.0);
        let y = oy + height - 1 - (normalized * (height as f64 - 1.0)).round() as isize;
        canvas.set(ox + index as isize, y, '*');
    }
}
