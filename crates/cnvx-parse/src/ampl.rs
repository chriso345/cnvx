use cnvx_lp::LpModel;

use super::LanguageParser;

#[derive(Default)]
pub struct AMPLLanguage;

impl AMPLLanguage {
    pub fn new() -> Self {
        Self {}
    }
}

impl LanguageParser for AMPLLanguage {
    fn parse(&self, _src: &str) -> Result<LpModel, String> {
        todo!("AMPL parser not implemented yet")
    }
}
