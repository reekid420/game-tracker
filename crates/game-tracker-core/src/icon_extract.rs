//! Icon extraction and image download utilities.

use std::fs;
use std::path::Path;

/// Extract the icon from a Windows `.exe` file and write it to `output_path`.
pub fn extract_exe_icon(
    exe_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(target_os = "windows")]
    {
        if let Some(parent) = Path::new(output_path).parent() {
            fs::create_dir_all(parent)?;
        }
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
pub async fn download_icon(
    url: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    fs::write(output_path, bytes)?;
    Ok(())
}
