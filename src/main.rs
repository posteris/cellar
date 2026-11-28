use reticuli::{diagnostics::profiler::Profiler, rendering::ansi::terminal::Terminal};
use std::io::Result;

fn main() -> Result<()> {
    let mut profiler = Profiler::new();
    let mut terminal = Terminal::new()?;
    profiler.bench(&mut terminal, 10_000)?;
    drop(terminal);

    for result in profiler.results() {
        println!("{}: {}us", result.0, result.1.as_micros());
    }

    Ok(())
}
