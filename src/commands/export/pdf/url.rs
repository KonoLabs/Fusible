use std::path::Path;

/// Returns the `file://` URL of `path`, percent-encoding every byte that is
/// not unreserved, so `#`, `?`, `%` and spaces stay part of the path.
pub(super) fn file_url(path: &Path) -> String {
    url_for(&path.to_string_lossy(), cfg!(windows))
}

fn drive_split(raw: &str) -> (String, String) {
    let plain = raw.strip_prefix(r"\\?\").unwrap_or(raw).replace('\\', "/");
    match plain.as_bytes() {
        [letter, b':', ..] if letter.is_ascii_alphabetic() => {
            (format!("/{}:", char::from(*letter)), plain[2..].to_string())
        }
        _ => (String::new(), plain),
    }
}

fn url_for(raw: &str, windows: bool) -> String {
    let (drive, rest) = if windows {
        drive_split(raw)
    } else {
        (String::new(), raw.to_string())
    };
    let encoded: String = rest
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                char::from(b).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect();
    format!("file://{drive}{encoded}")
}

#[cfg(test)]
mod tests {
    use super::{file_url, url_for};
    use std::path::Path;

    #[test]
    fn file_url_escapes_reserved_characters() {
        assert_eq!(
            file_url(Path::new("/tmp/a b#c?d%e/s.svg")),
            "file:///tmp/a%20b%23c%3Fd%25e/s.svg"
        );
    }

    #[test]
    fn windows_paths_become_drive_urls() {
        assert_eq!(
            url_for(r"C:\Users\me\a b\s.svg", true),
            "file:///C:/Users/me/a%20b/s.svg"
        );
        assert_eq!(url_for(r"\\?\D:\p\s.svg", true), "file:///D:/p/s.svg");
        assert_eq!(url_for(r"/tmp/a\b.svg", false), "file:///tmp/a%5Cb.svg");
    }
}
