use reticuli::rendering::{ansi::terminal::Terminal, renderer::Renderer};
use std::io::Result;

fn main() -> Result<()> {
    let duration = {
        let mut terminal = Terminal::new()?;
        terminal.bench(10_000)?
    };

    println!("render: {}us", duration.as_micros());
    Ok(())
}
