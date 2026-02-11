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

pub fn parse(contents: &str, file_type: &str) -> Result<Model, String> {
    let model = match file_type {
        "gmpl" => GMPLLanguage::new().parse(contents)?,
        "ampl" => AMPLLanguage::new().parse(contents)?,
        "mps" => MPSLanguage::new().parse(contents)?,
        _ => return Err(format!("unsupported file type: {}", file_type)),
    };

    Ok(model)
}
