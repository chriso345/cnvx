mod command;
mod lang;

mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::cli()
}
