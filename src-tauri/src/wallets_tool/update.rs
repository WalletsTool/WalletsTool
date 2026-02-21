use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Runtime};
use tauri_plugin_updater::UpdaterExt;

const GH_PROXY_BASE_URL: &str = "https://gh-proxy.org/";

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

#[derive(Debug, Serialize)]
pub struct UpdateCheckResult {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateDownloadProgress {
    pub status: String,
    pub progress: Option<u64>,
    pub total: Option<u64>,
    pub message: String,
}

fn to_gh_proxy_url(input: &str) -> String {
    let url = input.trim();
    if url.is_empty() {
        return String::new();
    }
    if url.starts_with(GH_PROXY_BASE_URL) {
        return url.to_string();
    }
    if url.starts_with("https://github.com/")
        || url.starts_with("https://api.github.com/")
        || url.starts_with("https://raw.githubusercontent.com/")
    {
        return format!("{GH_PROXY_BASE_URL}{url}");
    }
    url.to_string()
}

async fn fetch_github_release(
    client: &reqwest::Client,
    url: &str,
) -> Result<GitHubRelease, String> {
    let response = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("请求 GitHub Release 失败: {e}"))?;

    let response = response
        .error_for_status()
        .map_err(|e| format!("请求 GitHub Release 失败: {e}"))?;

    response
        .json()
        .await
        .map_err(|e| format!("解析 GitHub Release 失败: {e}"))
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
    let proxy_url = to_gh_proxy_url(&url);

    let client = reqwest::Client::builder()
        .user_agent(format!("WalletsTool/{current_version}"))
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {e}"))?;

    let mut used_proxy = false;
    let release = match fetch_github_release(&client, &url).await {
        Ok(release) => release,
        Err(first_error) => {
            used_proxy = true;
            fetch_github_release(&client, &proxy_url)
                .await
                .map_err(|second_error| format!("{first_error}; 代理重试失败: {second_error}"))?
        }
    };

    if release.draft {
        return Ok(None);
    }

    let latest_version = release.tag_name.trim().trim_start_matches('v').to_string();
    let has_update = is_newer_version(&current_version, &latest_version)?;

    if !has_update {
        return Ok(None);
    }

    let html_url = if used_proxy {
        to_gh_proxy_url(&release.html_url)
    } else {
        release.html_url
    };

    Ok(Some(GithubReleaseUpdateInfo {
        current_version,
        latest_version,
        html_url,
        name: release.name,
        body: release.body,
        published_at: release.published_at,
        prerelease: release.prerelease,
    }))
}

/// 使用 Tauri Updater 检查更新
#[command]
pub async fn check_update<R: Runtime>(
    app: AppHandle<R>,
    current_version: String,
) -> Result<UpdateCheckResult, String> {
    println!("[check_update] 开始检查更新, current_version: {}", current_version);
    
    println!("[check_update] 尝试获取 updater...");
    
    let updater = app
        .updater()
        .map_err(|e| {
            let err_msg = format!("获取更新器失败: {e}");
            println!("[check_update] 获取更新器错误: {}", err_msg);
            err_msg
        })?;

    println!("[check_update] 获取到 updater, 准备检查更新...");

    println!("[check_update] 调用 updater.check() 进行网络请求...");
    
    match updater.check().await {
        Ok(Some(update)) => {
            let latest_version = update.version.clone();
            let release_notes = update.body.clone();
            let download_url = update.download_url.to_string();
            
            println!("[check_update] 发现新版本: {} -> {}", current_version, latest_version);
            
            Ok(UpdateCheckResult {
                has_update: true,
                current_version,
                latest_version,
                release_notes,
                download_url: Some(download_url),
                published_at: update.date.map(|d| d.to_string()),
            })
        }
        Ok(None) => {
            println!("[check_update] 当前已是最新版本: {}", current_version);
            Ok(UpdateCheckResult {
                has_update: false,
                latest_version: current_version.clone(),
                current_version,
                release_notes: None,
                download_url: None,
                published_at: None,
            })
        }
        Err(e) => {
            let err_msg = format!("检查更新失败: {e}");
            println!("[check_update] 网络请求失败: {}", err_msg);
            Err(err_msg)
        }
    }
}

/// 下载并安装更新
#[command]
pub async fn download_and_install_update<R: Runtime>(
    app: AppHandle<R>,
) -> Result<String, String> {
    let updater = app
        .updater()
        .map_err(|e| format!("获取更新器失败: {e}"))?;

    match updater.check().await {
        Ok(Some(update)) => {
            // 下载更新
            let bytes = update
                .download(|_chunk_length, _content_length| {
                    // 可以在这里发送进度事件到前端
                }, || {
                    // 下载完成回调
                })
                .await
                .map_err(|e| format!("下载更新失败: {e}"))?;

            // 安装更新
            update
                .install(bytes)
                .map_err(|e| format!("安装更新失败: {e}"))?;

            Ok("更新下载完成，即将重启应用".to_string())
        }
        Ok(None) => Err("没有可用的更新".to_string()),
        Err(e) => Err(format!("检查更新失败: {e}")),
    }
}

/// 仅下载更新，不安装
#[command]
pub async fn download_update_only<R: Runtime>(
    app: AppHandle<R>,
) -> Result<String, String> {
    let updater = app
        .updater()
        .map_err(|e| format!("获取更新器失败: {e}"))?;

    match updater.check().await {
        Ok(Some(update)) => {
            // 下载更新
            let _bytes = update
                .download(|chunk_length, content_length| {
                    let progress = if let Some(total) = content_length {
                        format!("下载进度: {} / {} bytes", chunk_length, total)
                    } else {
                        format!("已下载: {} bytes", chunk_length)
                    };
                    println!("{}", progress);
                }, || {
                    println!("下载完成");
                })
                .await
                .map_err(|e| format!("下载更新失败: {e}"))?;

            Ok(format!("更新 v{} 下载完成", update.version))
        }
        Ok(None) => Err("没有可用的更新".to_string()),
        Err(e) => Err(format!("检查更新失败: {e}")),
    }
}
