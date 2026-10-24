const MAX_BYTES: usize = 64 * 1024;

/// Captured stderr of the sidecar, capped so a noisy process cannot grow it without bound.
#[derive(Debug, Default)]
pub struct Log {
    text: String,
}

impl Log {
    /// Appends one line, dropping the oldest text when the cap is exceeded.
    pub fn push(&mut self, line: &str) {
        self.text.push_str(line.trim_end());
        self.text.push('\n');
        if self.text.len() > MAX_BYTES {
            let cut = self.text.len() - MAX_BYTES;
            let boundary = (cut..self.text.len())
                .find(|index| self.text.is_char_boundary(*index))
                .unwrap_or(self.text.len());
            self.text.drain(..boundary);
        }
    }

    /// The captured text without its trailing newline.
    pub fn text(&self) -> String {
        self.text.trim_end().to_string()
    }

    /// Empties the log.
    pub fn clear(&mut self) {
        self.text.clear();
    }
}
