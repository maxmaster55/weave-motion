//! Terminal demo of the Weave pill-to-panel morph.
//!
//! A rounded button expands into a panel and back. The expansion can be
//! interrupted at any moment: the position and the velocity stay continuous,
//! the content is clipped by the surface, the text stays readable and the
//! animation goes idle once it settles.
//!
//! ```text
//! cargo run -p button-to-panel              # scripted acceptance run
//! cargo run -p button-to-panel -- --interactive
//! ```

mod interactive;
mod report;
mod surface;

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
    "weave button-to-panel demo

USAGE:
    button-to-panel              run the scripted acceptance scenario
    button-to-panel -i           run the interactive demo

INTERACTIVE COMMANDS:
    space        toggle between button and panel
    b / p        collapse / expand
    m            toggle reduced motion
    v <vx> <vy>  hand a velocity to the surface (like a released drag)
    q            quit"
}
