#![deny(clippy::all)]

mod config;
mod dataset;
mod rule;
mod rules;

pub use config::LintConfig;
pub use rule::LintRule;
pub use rules::{get_rules, LintParams};
