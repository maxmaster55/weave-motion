//! Error types shared by the motion primitives.

use core::fmt;

/// Errors produced when a motion primitive is configured with values that
/// cannot describe a physical system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpringError {
    /// `mass` was not a finite number.
    NonFiniteMass,
    /// `mass` was zero or negative.
    NonPositiveMass,
    /// `stiffness` was not a finite number.
    NonFiniteStiffness,
    /// `stiffness` was zero or negative.
    NonPositiveStiffness,
    /// `damping_ratio` was not a finite number.
    NonFiniteDamping,
    /// `damping_ratio` was negative.
    NegativeDamping,
    /// `position_epsilon` was not a finite number.
    NonFinitePositionEpsilon,
    /// `position_epsilon` was negative.
    NegativePositionEpsilon,
    /// `velocity_epsilon` was not a finite number.
    NonFiniteVelocityEpsilon,
    /// `velocity_epsilon` was negative.
    NegativeVelocityEpsilon,
}

impl fmt::Display for SpringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            SpringError::NonFiniteMass => "mass must be a finite number",
            SpringError::NonPositiveMass => "mass must be greater than zero",
            SpringError::NonFiniteStiffness => "stiffness must be a finite number",
            SpringError::NonPositiveStiffness => "stiffness must be greater than zero",
            SpringError::NonFiniteDamping => "damping ratio must be a finite number",
            SpringError::NegativeDamping => "damping ratio must not be negative",
            SpringError::NonFinitePositionEpsilon => "position epsilon must be a finite number",
            SpringError::NegativePositionEpsilon => "position epsilon must not be negative",
            SpringError::NonFiniteVelocityEpsilon => "velocity epsilon must be a finite number",
            SpringError::NegativeVelocityEpsilon => "velocity epsilon must not be negative",
        };
        f.write_str(message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SpringError {}
