//! The interactive part of the playground.
//!
//! Input is line based on purpose: raw terminal modes differ between
//! platforms and would make the demo unusable over SSH or inside CI. The
//! commands are typed and confirmed with Enter.

use std::io::{BufRead, Write};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use weave_demo_support::ansi;
use weave_demo_support::{Canvas, DemoClock};
use weaver_motion::{AnimationId, MotionEngine, MotionPolicy, MotionToken};

const WIDTH: usize = 72;
const HEIGHT: usize = 18;
const TRACE_SAMPLES: usize = 240;

struct State {
    engine: MotionEngine,
    id: AnimationId,
    token: MotionToken,
    target: f64,
    reduced: bool,
    slow_motion: bool,
    trace: Vec<f64>,
}

impl State {
    fn new() -> Self {
        let token = MotionToken::DefaultSpatial;
        let mut engine = MotionEngine::new();
        let id = engine.spring("value", 0.0, token);

        Self {
            engine,
            id,
            token,
            target: 1.0,
            reduced: false,
            slow_motion: false,
            trace: Vec::with_capacity(TRACE_SAMPLES),
        }
    }

    /// Rebuilds the animation with the current token and policy, keeping the
    /// value and the velocity of the running motion.
    fn apply_token(&mut self) {
        let policy = if self.reduced {
            MotionPolicy::Reduced
        } else {
            MotionPolicy::Full
        };
        self.engine.set_policy(policy);
        self.id = self.engine.spring("value", 0.0, self.token);
    }

    fn set_target(&mut self, target: f64) {
        self.target = target;
        self.engine.retarget(self.id, self.target);
    }

    fn value(&self) -> f64 {
        self.engine
            .value_by_key("value")
            .and_then(|value| value.as_scalar())
            .unwrap_or(0.0)
    }

    fn velocity(&self) -> f64 {
        self.engine
            .scheduler()
            .get_animated::<f64>(self.id)
            .map(|animated| animated.velocity())
            .unwrap_or(0.0)
    }
}

/// Reads commands from stdin in a background thread.
fn spawn_reader() -> Receiver<String> {
    let (sender, receiver) = channel();
    thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    receiver
}

/// Runs the interactive playground.
pub fn run() {
    let commands = spawn_reader();
    let mut state = State::new();
    state.apply_token();

    let clock = DemoClock::new();
    let mut stdout = std::io::stdout();
    let _ = write!(stdout, "{}{}", ansi::HIDE_CURSOR, ansi::CLEAR);
    let _ = stdout.flush();

    let mut running = true;

    while running {
        state.engine.tick(clock.elapsed());

        let value = state.value();
        state.trace.push(value);
        if state.trace.len() > TRACE_SAMPLES {
            state.trace.remove(0);
        }

        render(&state);

        match commands.recv_timeout(std::time::Duration::from_millis(16)) {
            Ok(command) => {
                running = handle_command(&mut state, command.trim());
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => running = false,
        }
    }

    let _ = write!(stdout, "{}{}", ansi::SHOW_CURSOR, ansi::RESET);
    let _ = stdout.flush();
}

/// Handles one command. Returns `false` when the playground should stop.
fn handle_command(state: &mut State, command: &str) -> bool {
    let mut parts = command.split_whitespace();
    let Some(verb) = parts.next() else {
        return true;
    };

    match verb {
        "q" | "quit" | "exit" => return false,
        "0" => state.set_target(0.0),
        "1" => state.set_target(1.0),
        "t" => {
            if let Some(value) = parts.next().and_then(|value| value.parse::<f64>().ok()) {
                state.set_target(value);
            }
        }
        "p" => {
            if let Some(name) = parts.next() {
                if let Some(token) = MotionToken::ALL.iter().find(|token| token.name() == name) {
                    state.token = *token;
                    state.apply_token();
                }
            }
        }
        "m" => {
            state.reduced = !state.reduced;
            state.apply_token();
        }
        "s" => state.slow_motion = !state.slow_motion,
        "r" => {
            state.trace.clear();
            state.set_target(state.target);
        }
        "?" | "h" | "help" => print_help(),
        _ => {}
    }

    true
}

fn print_help() {
    println!("commands: 0 | 1 | t <value> | p <token> | m | s | r | q");
}

/// Draws the current state.
fn render(state: &State) {
    let mut canvas = Canvas::new(WIDTH, HEIGHT);

    canvas.text(0, 0, "Weave spring playground");
    canvas.text(
        0,
        1,
        &format!(
            "token: {:<18} policy: {:<8} target: {:.2}   [?] for help, [q] to quit",
            state.token.name(),
            if state.reduced { "reduced" } else { "full" },
            state.target
        ),
    );

    canvas.hline(3, 0, WIDTH as isize - 1, '-');

    // Trace.
    let maximum = 1.4;
    for (index, value) in state.trace.iter().enumerate() {
        let x = index as isize;
        if x >= WIDTH as isize {
            break;
        }
        let normalized = (value / maximum).clamp(0.0, 1.0);
        let y = 4.0 + (HEIGHT as f64 - 5.0) * (1.0 - normalized);
        canvas.set(x, y.round() as isize, '*');
    }

    // Target line.
    let target_y = 4.0 + (HEIGHT as f64 - 5.0) * (1.0 - (state.target / maximum));
    canvas.hline(target_y.round() as isize, 0, WIDTH as isize - 1, '-');

    canvas.text(
        0,
        HEIGHT as isize - 1,
        &format!(
            "value={:.4}  velocity={:+.4}  samples={}  {}",
            state.value(),
            state.velocity(),
            state.trace.len(),
            if state.slow_motion { "slow motion" } else { "" }
        ),
    );

    let mut stdout = std::io::stdout();
    let _ = write!(stdout, "{}{}", ansi::HOME, ansi::RESET);
    let _ = writeln!(stdout, "{}", canvas.render());
    let _ = stdout.flush();
}
