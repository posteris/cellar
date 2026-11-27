use reticuli::{
    diagnostics::chaos::Chaos,
    rendering::{ansi::terminal::Terminal, renderer::Renderer},
};
use std::io::Result;

fn main() -> Result<()> {
    let mut chaos = Chaos::new();
    let mut terminal = Terminal::new()?;
    terminal.iterate(10_000, &mut chaos)?;
    let paint = terminal.metrics().paint.average().as_micros();
    let present = terminal.metrics().present.average().as_micros();
    let flush = terminal.metrics().flush.average().as_micros();
    drop(terminal);

    println!(
        "paint: {}us - present: {}us - flush: {}us",
        paint, present, flush,
    );
    Ok(())
}
