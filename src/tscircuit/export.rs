//! `export` invocation: formats the output path and reuses cached exports.

use std::path::{Path, PathBuf};

use super::command::export_command;
use super::{cache, process, resolve_hardware_context, Runner, RunnerError};

impl Runner {
    /// Runs the tscircuit export command.
    pub fn export(
        &self,
        file: &Path,
        format: &str,
        output: Option<&Path>,
    ) -> Result<(), RunnerError> {
        let abs_output = output.map(absolute_output);
        let cache = abs_output.as_ref().and_then(|_| self.cache_for(file));
        if let (Some(cache), Some(dest)) = (&cache, &abs_output) {
            if cache.restore(format, dest) {
                return Ok(());
            }
        }
        if let Some(parent) = abs_output.as_deref().and_then(Path::parent) {
            std::fs::create_dir_all(parent).ok();
        }

        let mut cmd = export_command(&self.kind, file, format, abs_output.as_deref());
        let out = cmd.output().map_err(|source| RunnerError::Io {
            command: process::format_command(&cmd),
            source,
        })?;

        if !out.status.success() {
            let details = process::extract_output_details(&out);
            return Err(RunnerError::CommandFailed {
                command: process::format_command(&cmd),
                code: out.status.code(),
                stderr: details.clone(),
                help: details,
            });
        }
        if let (Some(cache), Some(dest)) = (&cache, &abs_output) {
            cache.put(format, dest);
        }
        Ok(())
    }

    /// Opens the build cache of the workspace `file` belongs to.
    pub(super) fn cache_for(&self, file: &Path) -> Option<cache::Cache> {
        let (dir, rel) = resolve_hardware_context(file);
        let root = dir.unwrap_or_else(|| match file.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
            _ => PathBuf::from("."),
        });
        cache::Cache::open(&self.kind, &root, &rel)
    }
}

/// Resolves a relative output path against the current directory.
fn absolute_output(p: &Path) -> PathBuf {
    if p.is_absolute() {
        return p.to_path_buf();
    }
    std::env::current_dir()
        .map(|cd| cd.join(p))
        .unwrap_or_else(|_| p.to_path_buf())
}
