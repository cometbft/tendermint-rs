#![no_std]

extern crate alloc;

mod prelude;

pub mod errors;
pub mod operations;
pub mod options;
pub mod predicates;
pub mod types;
mod verifier;

pub use verifier::{PredicateVerifier, ProdVerifier, Verdict, Verifier};
