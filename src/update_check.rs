//! Update checking functionality using GitHub Releases API.
//!
//! Allows users to check if a newer version of Unfold is available.

use serde::Deserialize;
use semver::Version;

/// State for the update check dialog
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateCheckState {
    /// Not checking, dialog not shown
    None,
    /// Currently fetching from GitHub
    Checking,
    /// Update available with version string and release URL
    UpdateAvailable { version: String, release_url: String },
    /// Already on latest version
    UpToDate,
    /// Error occurred during check
    Error(String),
}

/// GitHub release API response (partial)
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub html_url: String,
}

/// Fetch the latest release from GitHub API and compare with current version
pub async fn fetch_latest_release() -> UpdateCheckState {
    const GITHUB_API_URL: &str = "https://api.github.com/repos/maumercado/unfold/releases/latest";
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

    // Build HTTP client with User-Agent (required by GitHub API)
    let client = match reqwest::Client::builder()
        .user_agent(format!("Unfold/{}", CURRENT_VERSION))
        .build()
    {
        Ok(c) => c,
        Err(e) => return UpdateCheckState::Error(format!("Failed to create HTTP client: {}", e)),
    };

    // Fetch latest release
    let response = match client.get(GITHUB_API_URL).send().await {
        Ok(r) => r,
        Err(e) => return UpdateCheckState::Error(format!("Network error: {}", e)),
    };

    // Check for HTTP errors
    if !response.status().is_success() {
        if response.status().as_u16() == 404 {
            return UpdateCheckState::Error("No releases found on GitHub".to_string());
        }
        return UpdateCheckState::Error(format!("GitHub API error: {}", response.status()));
    }

    // Parse JSON response
    let release: GitHubRelease = match response.json().await {
        Ok(r) => r,
        Err(e) => return UpdateCheckState::Error(format!("Failed to parse response: {}", e)),
    };

    // Parse versions (remove leading 'v' if present)
    let latest_version_str = release.tag_name.trim_start_matches('v');
    let current_version = match Version::parse(CURRENT_VERSION) {
        Ok(v) => v,
        Err(e) => return UpdateCheckState::Error(format!("Invalid current version: {}", e)),
    };
    let latest_version = match Version::parse(latest_version_str) {
        Ok(v) => v,
        Err(e) => return UpdateCheckState::Error(format!("Invalid release version '{}': {}", latest_version_str, e)),
    };

    // Compare versions
    if latest_version > current_version {
        UpdateCheckState::UpdateAvailable {
            version: release.tag_name,
            release_url: release.html_url,
        }
    } else {
        UpdateCheckState::UpToDate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_check_state_enum() {
        // Test that UpdateCheckState values are distinct and comparable
        assert_eq!(UpdateCheckState::None, UpdateCheckState::None);
        assert_eq!(UpdateCheckState::Checking, UpdateCheckState::Checking);
        assert_eq!(UpdateCheckState::UpToDate, UpdateCheckState::UpToDate);

        assert_ne!(UpdateCheckState::None, UpdateCheckState::Checking);
        assert_ne!(UpdateCheckState::Checking, UpdateCheckState::UpToDate);

        // Test UpdateAvailable variant
        let update1 = UpdateCheckState::UpdateAvailable {
            version: "v1.0.0".to_string(),
            release_url: "https://github.com/test".to_string(),
        };
        let update2 = UpdateCheckState::UpdateAvailable {
            version: "v1.0.0".to_string(),
            release_url: "https://github.com/test".to_string(),
        };
        assert_eq!(update1, update2);

        // Test Error variant
        let error1 = UpdateCheckState::Error("Network error".to_string());
        let error2 = UpdateCheckState::Error("Network error".to_string());
        assert_eq!(error1, error2);
        assert_ne!(UpdateCheckState::Error("a".to_string()), UpdateCheckState::Error("b".to_string()));
    }
}
