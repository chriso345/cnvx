mod lp;
mod mps;

use std::collections::HashMap;
use std::path::Path;

use cnvx_core::{CnvxError, Model, Var};

/// Reading and writing [`Model`]s to and from `.lp` and `.mps` files.
///
/// # Examples
///
/// ```rust,no_run
/// # use cnvx_core::Model;
/// # use cnvx_lp::LpModelIo;
/// # fn example(model: &Model) -> Result<(), cnvx_core::CnvxError> {
/// model.write("diet.lp")?;
/// model.write_mps("diet.mps")?;
/// let round_tripped = Model::read("diet.lp")?;
/// # Ok(())
/// # }
/// ```
pub trait LpModelIo {
    /// Reads a model from `path`, inferring the format from its
    /// extension (`.lp` or `.mps`).
    fn read(path: impl AsRef<Path>) -> Result<Model, CnvxError>;

    /// Writes this model to `path`, inferring the format from its
    /// extension (`.lp` or `.mps`).
    fn write(&self, path: impl AsRef<Path>) -> Result<(), CnvxError>;

    fn read_lp(path: impl AsRef<Path>) -> Result<Model, CnvxError>;
    fn write_lp(&self, path: impl AsRef<Path>) -> Result<(), CnvxError>;
    fn read_mps(path: impl AsRef<Path>) -> Result<Model, CnvxError>;
    fn write_mps(&self, path: impl AsRef<Path>) -> Result<(), CnvxError>;
}

impl LpModelIo for Model {
    fn read(path: impl AsRef<Path>) -> Result<Model, CnvxError> {
        let path = path.as_ref();
        match extension_of(path) {
            Some("lp") => Model::read_lp(path),
            Some("mps") => Model::read_mps(path),
            other => Err(unrecognized_extension(other)),
        }
    }

    fn write(&self, path: impl AsRef<Path>) -> Result<(), CnvxError> {
        let path = path.as_ref();
        match extension_of(path) {
            Some("lp") => self.write_lp(path),
            Some("mps") => self.write_mps(path),
            other => Err(unrecognized_extension(other)),
        }
    }

    fn read_lp(path: impl AsRef<Path>) -> Result<Model, CnvxError> {
        let path = path.as_ref();
        let contents = read_to_string(path)?;
        lp::parse(&contents)
    }

    fn write_lp(&self, path: impl AsRef<Path>) -> Result<(), CnvxError> {
        write_string(path.as_ref(), lp::render(self))
    }

    fn read_mps(path: impl AsRef<Path>) -> Result<Model, CnvxError> {
        let path = path.as_ref();
        let contents = read_to_string(path)?;
        mps::parse(&contents)
    }

    fn write_mps(&self, path: impl AsRef<Path>) -> Result<(), CnvxError> {
        write_string(path.as_ref(), mps::render(self))
    }
}

fn extension_of(path: &Path) -> Option<&str> {
    path.extension().and_then(|e| e.to_str())
}

fn unrecognized_extension(extension: Option<&str>) -> CnvxError {
    CnvxError::Numerical(format!(
        "unrecognized model file extension {extension:?} (expected .lp or .mps)"
    ))
}

fn read_to_string(path: &Path) -> Result<String, CnvxError> {
    std::fs::read_to_string(path).map_err(|e| {
        CnvxError::Numerical(format!("could not read {}: {e}", path.display()))
    })
}

fn write_string(path: &Path, contents: String) -> Result<(), CnvxError> {
    std::fs::write(path, contents).map_err(|e| {
        CnvxError::Numerical(format!("could not write {}: {e}", path.display()))
    })
}

pub(super) fn var_names(model: &Model) -> HashMap<Var, String> {
    model
        .vars()
        .enumerate()
        .map(|(i, v)| {
            let name = model
                .var_name(v)
                .ok()
                .flatten()
                .map(str::to_string)
                .unwrap_or_else(|| format!("x{i}"));
            (v, name)
        })
        .collect()
}
