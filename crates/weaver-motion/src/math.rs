//! Small math helpers so the crate can be built without `std`.
//!
//! `std` provides hardware-accelerated `exp`/`sqrt`; `no_std` builds fall
//! back to the pure-Rust [`libm`] crate, which produces bit-identical
//! results on every target.

#[cfg(all(not(feature = "std"), not(feature = "libm")))]
compile_error!(
    "`weaver-motion` built without the `std` feature needs the `libm` feature for \
     floating point math: use `--no-default-features --features libm`"
);

#[cfg(feature = "std")]
#[inline]
pub(crate) fn exp(x: f64) -> f64 {
    x.exp()
}

#[cfg(not(feature = "std"))]
#[inline]
pub(crate) fn exp(x: f64) -> f64 {
    libm::exp(x)
}

#[cfg(feature = "std")]
#[inline]
pub(crate) fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

#[cfg(not(feature = "std"))]
#[inline]
pub(crate) fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}

#[cfg(feature = "std")]
#[inline]
pub(crate) fn abs(x: f64) -> f64 {
    x.abs()
}

#[cfg(not(feature = "std"))]
#[inline]
pub(crate) fn abs(x: f64) -> f64 {
    libm::fabs(x)
}

#[cfg(feature = "std")]
#[inline]
pub(crate) fn cos(x: f64) -> f64 {
    x.cos()
}

#[cfg(not(feature = "std"))]
#[inline]
pub(crate) fn cos(x: f64) -> f64 {
    libm::cos(x)
}

#[cfg(feature = "std")]
#[inline]
pub(crate) fn sin(x: f64) -> f64 {
    x.sin()
}

#[cfg(not(feature = "std"))]
#[inline]
pub(crate) fn sin(x: f64) -> f64 {
    libm::sin(x)
}

#[cfg(feature = "std")]
#[inline]
pub(crate) fn ln(x: f64) -> f64 {
    x.ln()
}

#[cfg(not(feature = "std"))]
#[inline]
pub(crate) fn ln(x: f64) -> f64 {
    libm::log(x)
}

#[cfg(feature = "std")]
#[inline]
pub(crate) fn floor(x: f64) -> f64 {
    x.floor()
}

#[cfg(not(feature = "std"))]
#[inline]
pub(crate) fn floor(x: f64) -> f64 {
    libm::floor(x)
}
