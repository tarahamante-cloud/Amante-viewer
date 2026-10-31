//! User-reviewed diagnostics. The model deliberately has no access to login,
//! session or chat state; file collection and delivery are separate modules.

pub mod files;
pub mod github;
mod model;
pub mod privacy;

pub use model::{Field, Report};
