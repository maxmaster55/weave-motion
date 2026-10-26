//! Interactive terminal playground for the Weave spring solver.
//!
//! Run it without arguments to print a report that compares the motion
//! presets, shows a trajectory, demonstrates retargeting and proves that
//! the result does not depend on the frame rate:
//!
//! ```text
//! cargo run -p spring-playground
//! ```
//!
//! Run it with `--interactive` for a live playground:
//!
//! ```text
//! cargo run -p spring-playground -- --interactive
//! ```
//!
//! Interactive commands (line based, press Enter after each one):
//!
//! | Command | Effect |
//! |---|---|
//! | `0` / `1` | move to 0 / 1 |
//! | `t <value>` | move to an arbitrary value |
//! | `p <token>` | switch preset (`fast_spatial`, `default_spatial`, …) |
//! | `m` | toggle reduced motion |
//! | `s` | toggle slow motion |
//! | `r` | clear the trace |
//! | `q` | quit |

mod interactive;
mod report;
mod trace;

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    if arguments
        .iter()
        .any(|argument| argument == "-h" || argument == "--help")
    {
        println!("{}", help());
        return;
    }

    if arguments
        .iter()
        .any(|argument| argument == "-i" || argument == "--interactive")
    {
        interactive::run();
        return;
    }

    report::run();
}

fn help() -> &'static str {
    "weave spring playground

USAGE:
    spring-playground              print the motion report
    spring-playground -i           run the interactive playground

INTERACTIVE COMMANDS:
    0 | 1            move to 0 or 1
    t <value>        move to an arbitrary value
    p <token>        switch preset (fast_spatial, default_spatial,
                     expressive_spatial, gentle_spatial)
    m                toggle reduced motion
    s                toggle slow motion
    r                clear the trace
    q                quit"
}
