use pyo3::prelude::*;

mod lp;

#[pymodule]
fn cnvx(m: &Bound<'_, PyModule>) -> PyResult<()> {
    lp::register(m)?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
