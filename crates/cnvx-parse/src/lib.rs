//! # CNVX Parse
//!
//! This crate provides simple parsers for various LP file formats, including AMPL, GMPL, and MPS,
//! built on top of [`cnvx_core`]. It defines a common [`LanguageParser`] trait and specific parsers for
//! each format.
//!
//! # Modules
//!
//! - [`ampl`]: Parser for AMPL format.
//! - [`gmpl`]: Parser for GMPL format.
//! - [`mps`]: Parser for MPS format.

pub mod ampl;
pub mod gmpl;
pub mod mps;

pub use ampl::AMPLLanguage;
use cnvx_lp::LpModel;
pub use gmpl::GMPLLanguage;
pub use mps::MPSLanguage;

/// Trait for parsers
pub trait LanguageParser {
    fn parse(&self, src: &str) -> Result<LpModel, String>;
}

pub fn parse(contents: &str, file_type: &str) -> Result<LpModel, String> {
    let model = match file_type {
        "ampl" => AMPLLanguage::new().parse(contents)?,
        "gmpl" => GMPLLanguage::new().parse(contents)?,
        "mps" => MPSLanguage::new().parse(contents)?,
        _ => return Err(format!("unsupported file type: {}", file_type)),
    };

    Ok(model)
}
