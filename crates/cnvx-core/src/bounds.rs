/// Variable bounds.
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::Bounds;
/// let new_bounds = Bounds::new(0.0, 1.0);
/// let fixed = Bounds { lower: 5.0, upper: 5.0 };
///
/// let from_ = Bounds::from(0.0..);
/// let to = Bounds::from(..1.0);
/// let to_inclusive = Bounds::from(..=1.0);
/// let inclusive = Bounds::from(0.0..=1.0);
/// let range = Bounds::from(0.0..1.0);
/// let full = Bounds::from(..);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub lower: f64,
    pub upper: f64,
}

impl Bounds {
    /// Creates a new `Bounds` instance with the given lower and upper bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_core::Bounds;
    /// let b = Bounds::new(0.0, 1.0);
    /// ```
    pub fn new(lower: f64, upper: f64) -> Self {
        Bounds { lower, upper }
    }

    /// Creates a new `Bounds` instance representing a fixed value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_core::Bounds;
    /// let b = Bounds::fixed(5.0);
    /// ```
    pub fn fixed(value: f64) -> Self {
        Bounds { lower: value, upper: value }
    }
}

/// ```rust
/// # use cnvx_core::Bounds;
/// let b = Bounds::from(0.0..);
/// ```
impl From<std::ops::RangeFrom<f64>> for Bounds {
    fn from(range: std::ops::RangeFrom<f64>) -> Self {
        Bounds { lower: range.start, upper: f64::INFINITY }
    }
}

/// ```rust
/// # use cnvx_core::Bounds;
/// let b = Bounds::from(..1.0);
/// ```
impl From<std::ops::RangeTo<f64>> for Bounds {
    fn from(range: std::ops::RangeTo<f64>) -> Self {
        Bounds { lower: f64::NEG_INFINITY, upper: range.end }
    }
}

/// ```rust
/// # use cnvx_core::Bounds;
/// let b = Bounds::from(..=1.0);
/// ```
impl From<std::ops::RangeToInclusive<f64>> for Bounds {
    fn from(range: std::ops::RangeToInclusive<f64>) -> Self {
        Bounds { lower: f64::NEG_INFINITY, upper: range.end }
    }
}

/// ```rust
/// # use cnvx_core::Bounds;
/// let b = Bounds::from(0.0..=1.0);
/// ```
impl From<std::ops::RangeInclusive<f64>> for Bounds {
    fn from(range: std::ops::RangeInclusive<f64>) -> Self {
        Bounds { lower: *range.start(), upper: *range.end() }
    }
}

/// ```rust
/// # use cnvx_core::Bounds;
/// let b = Bounds::from(0.0..1.0);
/// ```
impl From<std::ops::Range<f64>> for Bounds {
    fn from(range: std::ops::Range<f64>) -> Self {
        Bounds { lower: range.start, upper: range.end }
    }
}

/// ```rust
/// # use cnvx_core::Bounds;
/// let b = Bounds::from(..);
/// ```
impl From<std::ops::RangeFull> for Bounds {
    fn from(_: std::ops::RangeFull) -> Self {
        Bounds { lower: f64::NEG_INFINITY, upper: f64::INFINITY }
    }
}
