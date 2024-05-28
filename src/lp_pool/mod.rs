//● Use fixed-point decimals based on the u64 type for all of these parameters, instead of
//floating points.
//● Assume that the price is constant for simplicity.
//● Implement a math model in pure Rust; integration with blockchain or UI is not necessary.
//● Include unit tests for at least the most important functions.
//● Choose any implementation paradigm (such as OOP, functional programming, etc.)
//based on your preferences.

// TODO: cargo clippy --D clippy::pedantic
// cargo audit
// 7) write your feature the easy way, use clones and lots of mut... then refine it (using cargo clippy --D clippy::pedantic did I mention that yet?)

pub mod data;
pub mod error;
pub mod pool;

use crate::error::{Error, Result};

