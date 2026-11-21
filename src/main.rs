use cellar::rendering::{ansi::terminal::Terminal, renderer::Renderer};
use std::io::Result;

fn main() -> Result<()> {
    let durations = {
        let mut terminal = Terminal::new()?;
        terminal.bench(10_000)?
    };

    durations
        .iter()
        .for_each(|d| println!("{}us", d.as_micros()));

    Ok(())
}
