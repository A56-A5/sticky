use sticky::cli;
use sticky::tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        cli::run()?;
    } else {
        tui::run()?;
    }

    Ok(())
}
