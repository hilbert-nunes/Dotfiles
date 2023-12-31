//! Rust platform registry: provides programmatic access to information about valid Rust platforms
//!
//! This crate provides an interface to the platform data canonically sourced
//! from the Rust compiler:
//!
//! <https://doc.rust-lang.org/nightly/rustc/platform-support.html>
//!
//! ## Minimum Supported Rust Version
//!
//! Rust **1.40** or higher.
//!
//! Minimum supported Rust version can be changed in the future, but it will be
//! done with a minor version bump.

#![no_std]
#![doc(html_root_url = "https://docs.rs/platforms/2.0.0-pre")]
#![forbid(unsafe_code)]
#![warn(missing_docs, unused_qualifications, rust_2018_idioms)]

#[cfg(feature = "std")]
extern crate std;

pub(crate) mod error;
pub mod platform;
pub mod target;

pub use crate::{
    error::Error,
    platform::{Platform, Tier},
    target::{TARGET_ARCH, TARGET_ENV, TARGET_OS},
};

#[cfg(feature = "std")]
pub use crate::platform::PlatformReq;
