//! "Check for Updates" against the GitHub Releases API.
//!
//! GPUI runs its own async executor, not Tokio, so the async `reqwest` client
//! would have no runtime here. Instead we use the blocking client and call it
//! from a GPUI background task (a thread pool), which keeps the UI responsive.

use semver::Version;
use serde::Deserialize;

const GITHUB_API_URL: &str = "https://api.github.com/repos/maumercado/unfold/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateCheck {
    Checking,
    UpdateAvailable {
        version: String,
        release_url: String,
    },
    UpToDate,
    Error(String),
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

/// Blocking: run on a background thread.
pub fn fetch_latest_release() -> UpdateCheck {
    let client = match reqwest::blocking::Client::builder()
        .user_agent(format!("Unfold/{}", CURRENT_VERSION))
        .build()
    {
        Ok(c) => c,
        Err(e) => return UpdateCheck::Error(format!("Failed to create HTTP client: {}", e)),
    };
    let response = match client.get(GITHUB_API_URL).send() {
        Ok(r) => r,
        Err(e) => return UpdateCheck::Error(format!("Network error: {}", e)),
    };
    if response.status().as_u16() == 404 {
        return UpdateCheck::Error("No releases found on GitHub".into());
    }
    if !response.status().is_success() {
        return UpdateCheck::Error(format!("GitHub API error: {}", response.status()));
    }
    match response.json::<GitHubRelease>() {
        Ok(release) => compare(CURRENT_VERSION, release),
        Err(e) => UpdateCheck::Error(format!("Failed to parse response: {}", e)),
    }
}

fn compare(current: &str, release: GitHubRelease) -> UpdateCheck {
    let latest_str = release.tag_name.trim_start_matches('v');
    let current = match Version::parse(current) {
        Ok(v) => v,
        Err(e) => return UpdateCheck::Error(format!("Invalid current version: {}", e)),
    };
    let latest = match Version::parse(latest_str) {
        Ok(v) => v,
        Err(e) => {
            return UpdateCheck::Error(format!("Invalid release version '{}': {}", latest_str, e));
        }
    };
    if latest > current {
        UpdateCheck::UpdateAvailable {
            version: release.tag_name,
            release_url: release.html_url,
        }
    } else {
        UpdateCheck::UpToDate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str) -> GitHubRelease {
        GitHubRelease {
            tag_name: tag.into(),
            html_url: "https://example.com".into(),
        }
    }

    #[test]
    fn newer_release_is_an_update() {
        assert!(matches!(
            compare("1.5.7", release("v1.6.0")),
            UpdateCheck::UpdateAvailable { .. }
        ));
        assert_eq!(compare("1.5.7", release("v1.5.7")), UpdateCheck::UpToDate);
        assert!(matches!(
            compare("1.5.7", release("nope")),
            UpdateCheck::Error(_)
        ));
    }
}
