//! Validated built-in pet packages and deterministic animation transitions.
mod animator;
mod package;

pub use animator::Animator;
pub use package::{Frame, Package, State};

#[cfg(test)]
mod tests;
