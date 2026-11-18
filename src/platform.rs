//! OS integration that GPUI does not cover: the CLI symlink and launching the
//! user's text editor.

use std::path::Path;
use std::process::Command;
#[cfg(unix)]
use std::{env, path::PathBuf};

/// Symlink this executable to /usr/local/bin/unfold, asking for an admin
/// password through the native macOS prompt.
#[cfg(target_os = "macos")]
pub fn install_cli_tool() -> Result<String, String> {
    let target_path = "/usr/local/bin/unfold";
    let exe_path =
        env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;
    if !PathBuf::from("/usr/local/bin").exists() {
        return Err("Directory /usr/local/bin does not exist. Please create it first.".into());
    }
    let script = format!(
        r#"do shell script "ln -sf '{}' '{}'" with administrator privileges"#,
        exe_path.to_string_lossy().replace('\'', "'\\''"),
        target_path
    );
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Failed to run osascript: {}", e))?;
    if output.status.success() {
        Ok("CLI installed! You can now use 'unfold' from the terminal.".into())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("canceled") {
            Err("Installation canceled.".into())
        } else {
            Err(format!("Failed to create symlink: {}", stderr.trim()))
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn install_cli_tool() -> Result<String, String> {
    use std::os::unix::fs::symlink;

    let target_path = PathBuf::from("/usr/local/bin/unfold");
    let exe_path =
        env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;
    if target_path.exists() || target_path.is_symlink() {
        let _ = std::fs::remove_file(&target_path);
    }
    match symlink(&exe_path, &target_path) {
        Ok(()) => Ok("CLI installed! You can now use 'unfold' from the terminal.".into()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Err(format!(
            "Permission denied. Run this command in terminal:\nsudo ln -sf \"{}\" \"{}\"",
            exe_path.display(),
            target_path.display()
        )),
        Err(e) => Err(format!("Failed to create symlink: {}", e)),
    }
}

#[cfg(not(unix))]
pub fn install_cli_tool() -> Result<String, String> {
    Err("CLI installation is only supported on Unix systems (macOS, Linux)".into())
}

/// Open the file in the default *text* editor (not the default JSON app,
/// which may be Unfold itself).
pub fn open_in_text_editor(path: &Path) {
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg("-t").arg(path).spawn();
    #[cfg(target_os = "windows")]
    let _ = Command::new("cmd")
        .args(["/C", "start", "", path.to_str().unwrap_or("")])
        .spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = Command::new("xdg-open").arg(path).spawn();
}
