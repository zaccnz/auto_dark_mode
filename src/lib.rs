mod args;

pub use crate::args::Args;

mod registry;

pub use crate::registry::{install, uninstall, update_theme, watch_registry};
