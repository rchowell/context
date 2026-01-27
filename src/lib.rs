// Allow some pedantic lints for prototype code
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::module_name_repetitions)]

pub mod cli;
pub mod core;
pub mod error;

pub use core::Cache;
pub use error::{ContextError, Result};
