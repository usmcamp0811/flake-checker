use crate::issue::Issue;

use std::env;

use sha2::{Digest, Sha256};

const TELEMETRY_ENDPOINT: &str = "https://install.determinate.systems/flake-checker/telemetry";

/// A telemetry report to identify trends in outdated locks against nixpkgs
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub(crate) struct TelemetryReport {
    pub distinct_id: String,

    pub version: String,
    pub is_ci: bool,

    pub disallowed: usize,
    pub outdated: usize,
    pub non_upstream: usize,
}

impl TelemetryReport {
    pub(crate) fn new(issues: &[Issue]) -> Result<TelemetryReport, env::VarError> {
        Ok(TelemetryReport {
            distinct_id: env::var("DETSYS_CORRELATION").or_else(|_| calculate_opaque_id())?,

            version: env!("CARGO_PKG_VERSION").to_string(),
            is_ci: is_ci::cached(),

            disallowed: issues
                .iter()
                .filter(|issue| issue.kind.is_disallowed())
                .count(),
            outdated: issues
                .iter()
                .filter(|issue| issue.kind.is_outdated())
                .count(),
            non_upstream: issues
                .iter()
                .filter(|issue| issue.kind.is_non_upstream())
                .count(),
        })
    }

    pub fn make_and_send(issues: &[Issue]) {
        if let Ok(report) = TelemetryReport::new(issues) {
            if let Ok(serialized) = serde_json::to_string_pretty(&report) {
                let _ = reqwest::blocking::Client::new()
                    .post(TELEMETRY_ENDPOINT)
                    .body(serialized)
                    .header("Content-Type", "application/json")
                    .timeout(std::time::Duration::from_millis(3000))
                    .send();
            }
        }
    }
}

fn calculate_opaque_id() -> Result<String, env::VarError> {
    let mut hasher = Sha256::new();
    hasher.update(env::var("GITHUB_REPOSITORY")?);
    hasher.update(env::var("GITHUB_REPOSITORY_ID")?);
    hasher.update(env::var("GITHUB_REPOSITORY_OWNER")?);
    hasher.update(env::var("GITHUB_REPOSITORY_OWNER_ID")?);

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
