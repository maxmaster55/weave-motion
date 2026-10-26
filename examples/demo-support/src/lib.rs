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
}

impl Canvas {
    /// Creates an empty canvas.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![' '; width * height],
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

    /// Clears the canvas.
    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|cell| *cell = ' ');
    }

    /// Writes a character, ignoring out-of-range coordinates.
    pub fn set(&mut self, x: isize, y: isize, ch: char) {
        if x < 0 || y < 0 {
            return;
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
        if width == 0 || height == 0 {
            return;
        }

        let max_radius = width.min(height) / 2;
        let radius = radius.min(max_radius);
        if radius == 0 {
            self.hline(y, x, x + width as isize - 1, ch);
            self.hline(y + height as isize - 1, x, x + width as isize - 1, ch);
            self.vline(x, y, y + height as isize - 1, ch);
            self.vline(x + width as isize - 1, y, y + height as isize - 1, ch);
            return;
        }

        let radius = radius as isize;
        let last_x = x + width as isize - 1;
        let last_y = y + height as isize - 1;

        for row in 0..height as isize {
            for column in 0..width as isize {
                let px = x + column;
                let py = y + row;

                // Distance from the corner centres decides whether the pixel
                // belongs to the rounded corner.
                let (cx, cy) = match (column < radius, row < radius) {
                    (true, true) => (x + radius - 1, y + radius - 1),
                    (true, false) => {
                        if last_y - (y + row) < radius {
                            (x + radius - 1, last_y - radius + 1)
                        } else {
                            (x + radius - 1, y + radius - 1)
                        }
                    }
                    (false, true) => {
                        if last_x - (x + column) < radius {
                            (last_x - radius + 1, y + radius - 1)
                        } else {
                            (x + radius - 1, y + radius - 1)
                        }
                    }
                    (false, false) => {
                        if last_x - (x + column) < radius && last_y - (y + row) < radius {
                            (last_x - radius + 1, last_y - radius + 1)
                        } else {
                            (x + radius - 1, y + radius - 1)
                        }
                    }
                };

                let dx = px - cx;
                let dy = py - cy;
                if dx * dx + dy * dy <= radius * radius {
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
