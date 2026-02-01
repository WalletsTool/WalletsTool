use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    name: Option<String>,
    body: Option<String>,
    draft: bool,
    prerelease: bool,
    published_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GithubReleaseUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub html_url: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub published_at: Option<String>,
    pub prerelease: bool,
}

fn parse_semver_triplet(input: &str) -> Result<(u32, u32, u32), String> {
    let trimmed = input.trim().trim_start_matches('v');
    let core = trimmed
        .split_once('-')
        .map(|(left, _)| left)
        .unwrap_or(trimmed);

    let mut parts = core.split('.');
    let major = parts
        .next()
        .ok_or_else(|| format!("无法解析版本号: {input}"))?
        .parse::<u32>()
        .map_err(|_| format!("无法解析版本号: {input}"))?;
    let minor = parts
        .next()
        .unwrap_or("0")
        .parse::<u32>()
        .map_err(|_| format!("无法解析版本号: {input}"))?;
    let patch = parts
        .next()
        .unwrap_or("0")
        .parse::<u32>()
        .map_err(|_| format!("无法解析版本号: {input}"))?;
    Ok((major, minor, patch))
}

fn is_newer_version(current: &str, latest: &str) -> Result<bool, String> {
    let current_triplet = parse_semver_triplet(current)?;
    let latest_triplet = parse_semver_triplet(latest)?;
    Ok(latest_triplet > current_triplet)
}

#[command]
pub async fn check_github_release_update(
    owner: Option<String>,
    repo: Option<String>,
    current_version: String,
) -> Result<Option<GithubReleaseUpdateInfo>, String> {
    let owner = owner.unwrap_or_else(|| "WalletsTool".to_string());
    let repo = repo.unwrap_or_else(|| "WalletsTool".to_string());

    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");

    let client = reqwest::Client::builder()
        .user_agent(format!("WalletsTool/{current_version}"))
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {e}"))?;

    let response = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("请求 GitHub Release 失败: {e}"))?;

    let response = response
        .error_for_status()
        .map_err(|e| format!("请求 GitHub Release 失败: {e}"))?;

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("解析 GitHub Release 失败: {e}"))?;

    if release.draft {
        return Ok(None);
    }

    let latest_version = release.tag_name.trim().trim_start_matches('v').to_string();
    let has_update = is_newer_version(&current_version, &latest_version)?;

    if !has_update {
        return Ok(None);
    }

    Ok(Some(GithubReleaseUpdateInfo {
        current_version,
        latest_version,
        html_url: release.html_url,
        name: release.name,
        body: release.body,
        published_at: release.published_at,
        prerelease: release.prerelease,
    }))
}
