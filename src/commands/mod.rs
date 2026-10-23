//! CLI subcommands implementation.

pub mod add;
pub mod body;
pub mod build;
pub mod check;
mod check_merge;
pub mod dev;
pub mod diff;
pub mod doctor;
pub mod export;
pub mod init;
pub mod install;
pub mod route;
pub mod search;
pub mod source;
pub mod sync;
pub mod target;

#[cfg(test)]
mod check_merge_tests;
#[cfg(test)]
mod tests;
