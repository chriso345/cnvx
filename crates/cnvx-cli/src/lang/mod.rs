pub mod ampl;
pub mod gmpl;
pub mod mps;

pub use ampl::AMPLLanguage;
use cnvx_core::Model;
pub use gmpl::GMPLLanguage;
pub use mps::MPSLanguage;

/// Trait for parsers
pub trait LanguageParser {
    fn parse(&self, src: &str) -> Result<Model, String>;
}
