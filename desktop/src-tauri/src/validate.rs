use crate::model::Settings;

const THEMES: &[&str] = &["system", "dark", "light"];
const FORBIDDEN: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// Checks a project name, returning the trimmed name or a message for the user.
pub fn project_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Give the project a name.".into());
    }
    if name == "." || name == ".." || name.ends_with('.') {
        return Err("The name cannot be \".\", \"..\" or end with a dot.".into());
    }
    if name.contains(FORBIDDEN) || name.chars().any(char::is_control) {
        return Err("The name cannot contain / \\ : * ? \" < > | characters.".into());
    }
    Ok(name.to_string())
}

/// Checks settings, returning a cleaned copy or a message for the user.
pub fn settings(input: Settings) -> Result<Settings, String> {
    if input.projects_dir.trim().is_empty() {
        return Err("Choose a projects folder.".into());
    }
    if input.port < 1024 {
        return Err("The studio port must be between 1024 and 65535.".into());
    }
    if !THEMES.contains(&input.theme.as_str()) {
        return Err("The theme must be system, dark or light.".into());
    }
    let currency = input.default_currency.trim().to_uppercase();
    if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("The currency must be a three-letter code such as EUR.".into());
    }
    Ok(Settings {
        default_currency: currency,
        projects_dir: input.projects_dir.trim().to_string(),
        ..input
    })
}
