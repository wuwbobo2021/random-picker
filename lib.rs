//! Generates random choices based on the weight table of probabilities.
//! It can be used to calculate each item's probability of being picked up
//! when picking a given amount of non-repetitive items, or to compare
//! the speed of OS random source with that of the CSPRNG.

// by wuwbobo2021 <https://github.com/wuwbobo2021>, <wuwbobo@outlook.com>

mod calc;
mod config;
mod picker;

pub use crate::{config::*, picker::*};

/// Possible errors returned by functions in this crate.
#[derive(Debug)]
pub enum Error {
    /// The table is invalid and cannot be used by the picker.
    InvalidTable,
    /// The given amount exceeds the amount of possible items in the table.
    InvalidAmount,
    /// Error from the random generator.
    RandError(rand::Error),
    /// Failure of the multi-thread probability calculator.
    ThreadError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            InvalidTable => write!(f, "Invalid probability table"),
            InvalidAmount => write!(f, "Invalid amount of items to be picked up"),
            RandError(e) => write!(f, "RNG Error: {:?}", e),
            ThreadError => write!(f, "Thread error during calculation"),
        }
    }
}
