//! The Tendermint P2P stack.

#![forbid(unsafe_code)]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic,
    clippy::unwrap_used,
    missing_docs,
    unused_import_braces,
    unused_qualifications
)]
#![doc(
    html_root_url = "https://docs.rs/tendermint-p2p/0.1.0",
    html_logo_url = "https://raw.githubusercontent.com/informalsystems/tendermint-rs/master/img/logo-tendermint-rs_3961x4001.png"
)]

pub mod error;
pub mod message;
pub mod peer;
pub mod secret_connection;
pub mod supervisor;
pub mod transport;
