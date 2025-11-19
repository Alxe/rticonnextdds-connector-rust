//! This is a fake binary crate to allow code snippets in documentation
//! to be tested as part of the regular Cargo build process.
//!
//! The intended usage is adding individual snippets as modules under this
//! `snippets` module, and then referencing them from the documentation using
//! `include_str!` macros.
//!
//! ## Example
//!
//! ```rust,no_run
//! //! This is my crate. My crate is amazing.
//! //!
//! //! ## Example code snippet
//! #![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/snippets/my_snippet.rs"))]
//! ```

// Effectively, this mod is not intended to be run as a binary, as all code is
// dead code. Hence, we add an allow attribute to suppress warnings.
#![allow(dead_code)]

fn main() {
    unimplemented!("This is a fake binary crate for documentation snippets only.");
}

mod connector;
mod input;
mod output;
mod quickstart;
