//! .exe icon extraction for Windows PC games.
//!
//! Uses `exeico` to extract icons from executables; saves to `static/icons/`.
//! `download_icon` fetches remote cover art from URLs (e.g. RAWG).

use std::fs;

/// Extract the icon from a Windows `.exe` file and write it to `output_path`.
///
/// Only available on Windows; returns an error on other platforms.
pub fn extract_exe_icon(exe_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(target_os = "windows")]
    {
        let icon_data = exeico::get_exe_ico(exe_path)?;
        fs::write(output_path, icon_data)?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (exe_path, output_path);
        Err("Icon extraction only supported on Windows".into())
    }
}

/// Download an image from `url` and save it to `output_path`.
pub async fn download_icon(url: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    fs::write(output_path, bytes)?;
    Ok(())
}
