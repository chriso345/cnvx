mod lang;
mod solve;
mod style;

mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::cli()
}
