//! The interactive part of the demo.
//!
//! Input is line based so that the demo works over SSH, in CI and on
//! every platform without touching raw terminal modes.

use std::io::{BufRead, Write};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use weave_core::MotionPolicy;
use weave_demo_support::ansi;
use weave_demo_support::{Canvas, DemoClock};

use crate::surface::Demo;

const WIDTH: usize = 76;
const HEIGHT: usize = 22;

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

/// Runs the interactive demo.
pub fn run() {
    let commands = spawn_reader();
    let mut demo = Demo::new();
    let clock = DemoClock::new();

    let mut stdout = std::io::stdout();
    let _ = write!(stdout, "{}{}", ansi::HIDE_CURSOR, ansi::CLEAR);
    let _ = stdout.flush();

    let mut running = true;

    while running {
        demo.tick(clock.elapsed());
        render(&demo);

        match commands.recv_timeout(std::time::Duration::from_millis(16)) {
            Ok(command) => running = handle_command(&mut demo, command.trim()),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => running = false,
        }
    }

    let _ = write!(stdout, "{}{}", ansi::SHOW_CURSOR, ansi::RESET);
    let _ = stdout.flush();
}

/// Handles one command. Returns `false` when the demo should stop.
fn handle_command(demo: &mut Demo, command: &str) -> bool {
    let mut parts = command.split_whitespace();
    let Some(verb) = parts.next() else {
        return true;
    };

    match verb {
        "q" | "quit" | "exit" => return false,
        "space" | "t" | "toggle" => demo.toggle(),
        "b" => demo.set_expanded(false),
        "p" => demo.set_expanded(true),
        "m" => {
            let policy = if demo.policy() == MotionPolicy::Reduced {
                MotionPolicy::Full
            } else {
                MotionPolicy::Reduced
            };
            demo.set_policy(policy);
        }
        "v" => {
            // Hand a velocity to the surface, like a released drag.
            let vx = parts
                .next()
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            let vy = parts
                .next()
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            demo.fling(vx, vy);
        }
        "?" | "h" | "help" => print_help(),
        _ => {}
    }

    true
}

fn print_help() {
    println!("commands: space | b | p | m | v <vx> <vy> | q");
}

/// Draws the demo.
fn render(demo: &Demo) {
    let mut canvas = Canvas::new(WIDTH, HEIGHT);

    canvas.text(0, 0, "Weave button-to-panel demo");
    canvas.text(
        0,
        1,
        "[space] toggle  [b] button  [p] panel  [m] reduced motion  [v vx vy] fling  [q] quit",
    );
    canvas.hline(3, 0, WIDTH as isize - 1, '-');

    demo.draw(&mut canvas);

    canvas.text(0, HEIGHT as isize - 1, &demo.status());

    let mut stdout = std::io::stdout();
    let _ = write!(stdout, "{}{}", ansi::HOME, ansi::RESET);
    let _ = writeln!(stdout, "{}", canvas.render());
    let _ = stdout.flush();
}
