//! Platform-specific abstractions.

/// Get the platform data directory
pub fn get_data_dir() -> anyhow::Result<std::path::PathBuf> {
    let dir = dirs::data_local_dir()
        .map(|d| d.join("MetallicMini"))
        .unwrap_or_else(|| std::path::PathBuf::from("./metallic-mini-data"));
    Ok(dir)
}

/// Get the platform config directory
pub fn get_config_dir() -> anyhow::Result<std::path::PathBuf> {
    let dir = dirs::config_dir()
        .map(|d| d.join("MetallicMini"))
        .unwrap_or_else(|| std::path::PathBuf::from("./metallic-mini-config"));
    Ok(dir)
}

/// Get the platform log directory
pub fn get_log_dir() -> anyhow::Result<std::path::PathBuf> {
    let data = get_data_dir()?;
    Ok(data.join("logs"))
}
