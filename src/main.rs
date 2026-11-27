use reticuli::{
    diagnostics::{chaos::Chaos, monitor::Monitor},
    rendering::{ansi::terminal::Terminal, renderer::Renderer},
};
use std::io::Result;

fn main() -> Result<()> {
    let mut monitor = Monitor::new();

    let mut chaos = Chaos::new();
    let mut terminal = Terminal::new()?;
    terminal.iterate(10_000, &mut chaos, &mut monitor)?;
    drop(terminal);

    println!(
        "paint: {}us - present: {}us - flush: {}us",
        monitor.watch(0).average().as_micros(),
        monitor.watch(1).average().as_micros(),
        monitor.watch(2).average().as_micros()
    );
    Ok(())
}
