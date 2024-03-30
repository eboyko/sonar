use crate::command_line_parser::fetch_arguments;

pub(crate) struct Settings {
    pub(crate) path: String,
    pub(crate) url: String,
}

pub(crate) fn build() -> Result<Settings, String> {
    let preferences = match fetch_arguments() {
        Ok(arguments) => arguments,
        Err(message) => return Err(message)
    };

    let path = preferences.get("path").ok_or("Mandatory `path` argument missing")?;
    let url = preferences.get("url").ok_or("Mandatory `url` argument missing")?;

    Ok(Settings { path: path.to_string(), url: url.to_string() })
}

