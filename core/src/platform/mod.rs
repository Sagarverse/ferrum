//! Platform-specific abstractions.

/// Get the platform data directory
pub fn get_data_dir() -> anyhow::Result<std::path::PathBuf> {
    let dir = dirs::data_local_dir()
        .map(|d| d.join("Ferrum"))
        .unwrap_or_else(|| std::path::PathBuf::from("./ferrum-data"));
    Ok(dir)
}

/// Get the platform config directory
pub fn get_config_dir() -> anyhow::Result<std::path::PathBuf> {
    let dir = dirs::config_dir()
        .map(|d| d.join("Ferrum"))
        .unwrap_or_else(|| std::path::PathBuf::from("./ferrum-config"));
    Ok(dir)
}

/// Get the platform log directory
pub fn get_log_dir() -> anyhow::Result<std::path::PathBuf> {
    let data = get_data_dir()?;
    Ok(data.join("logs"))
}
