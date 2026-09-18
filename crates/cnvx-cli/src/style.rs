use clap::builder::styling::{AnsiColor, Effects, Style, Styles};

pub(crate) const HEADER: Style =
    AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);
pub(crate) const USAGE: Style = HEADER;
pub(crate) const LITERAL: Style = AnsiColor::Cyan.on_default();
pub(crate) const PLACEHOLDER: Style =
    AnsiColor::White.on_default().effects(Effects::ITALIC).dimmed();
pub(crate) const ERROR: Style = AnsiColor::BrightRed
    .on_default()
    .effects(Effects::BOLD)
    .effects(Effects::UNDERLINE);
pub(crate) const VALID: Style =
    AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);
pub(crate) const INVALID: Style =
    AnsiColor::BrightYellow.on_default().effects(Effects::BOLD).dimmed();
pub(crate) const RESET: &str = "\x1b[0m";

pub(crate) const CLI_STYLING: Styles = Styles::styled()
    .header(HEADER)
    .usage(USAGE)
    .literal(LITERAL)
    .placeholder(PLACEHOLDER)
    .error(ERROR)
    .valid(VALID)
    .invalid(INVALID);

pub fn after_help() -> String {
    format!(
        "See '{}{} help {}{}<command>{}' for more information on a specific command.",
        HEADER,
        env!("CARGO_BIN_NAME"),
        RESET,
        PLACEHOLDER,
        RESET,
    )
}
