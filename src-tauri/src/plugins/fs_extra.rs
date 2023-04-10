// https://github.com/tauri-apps/tauri-plugin-fs-extra/blob/dev/src/lib.rs

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{path::PathBuf, process::Command, time::{SystemTime, UNIX_EPOCH}};

use serde::{ser::Serializer, Serialize};
use tauri::command;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Permissions {
    readonly: bool,
    #[cfg(unix)]
    mode: u32,
}

#[cfg(unix)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UnixMetadata {
    dev: u64,
    ino: u64,
    mode: u32,
    nlink: u64,
    uid: u32,
    gid: u32,
    rdev: u64,
    blksize: u64,
    blocks: u64,
}

#[command]
pub async fn exists(path: PathBuf) -> bool {
    path.exists()
}

#[command]
pub fn open_file(path: &str) {
    #[cfg(target_os = "windows")]
    Command::new("explorer")
        .args(["/select,", path])
        .spawn()
        .unwrap();

    #[cfg(target_os = "macos")]
    Command::new("open").args(["-R", path]).spawn().unwrap();

    // https://askubuntu.com/a/31071
    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(path).spawn().unwrap();
}
