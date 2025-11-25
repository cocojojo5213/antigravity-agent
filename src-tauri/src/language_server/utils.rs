use std::cmp::min;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use regex::Regex;
use sysinfo::System;
use walkdir::WalkDir;
use read_process_memory as _;

#[cfg(target_os = "windows")]
use crate::language_server::windows::scan_process_for_token;

#[cfg(target_os = "linux")]
use crate::language_server::linux::scan_process_for_token;

#[cfg(target_os = "macos")]
use crate::language_server::macos::scan_process_for_token;

pub(crate) const SCAN_AHEAD: usize = 200;
pub(crate) const CHUNK_SIZE: usize = 512 * 1024; // 512KB 分块读取，降低单次读耗时
pub(crate) const MAX_REGION_BYTES: usize = 64 * 1024 * 1024; // 每个区域最多扫描 64MB，加速

/// 查找最新的 Antigravity.log（按修改时间）
pub fn find_latest_antigravity_log() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(dir) = dirs::data_dir() {
        candidates.push(dir.join("Antigravity").join("logs"));
    }
    if let Some(dir) = dirs::config_dir() {
        candidates.push(dir.join("Antigravity").join("logs"));
    }

    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;

    for root in candidates {
        if !root.exists() {
            continue;
        }
        if let Ok(entries) = WalkDir::new(root).max_depth(6).into_iter().collect::<Result<Vec<_>, _>>() {
            for entry in entries {
                let path = entry.path();
                if path.file_name().is_some_and(|n| n == "Antigravity.log") && path.is_file() {
                    if let Ok(meta) = path.metadata() {
                        if let Ok(modified) = meta.modified() {
                            match &newest {
                                Some((_, ts)) if *ts >= modified => {}
                                _ => newest = Some((path.to_path_buf(), modified)),
                            }
                        }
                    }
                }
            }
        }
    }

    newest.map(|(p, _)| p)
}

/// 从日志内容解析 HTTPS/HTTP/extension 端口
pub fn parse_ports_from_log(content: &str) -> (Option<u16>, Option<u16>, Option<u16>) {
    let https_re = Regex::new(r"random port at (\d+) for HTTPS").unwrap();
    let http_re = Regex::new(r"random port at (\d+) for HTTP").unwrap();
    let ext_re = Regex::new(r"extension server client at port (\d+)").unwrap();

    let https_port = https_re
        .captures_iter(content)
        .last()
        .and_then(|c| c.get(1)?.as_str().parse::<u16>().ok());
    let http_port = http_re
        .captures_iter(content)
        .last()
        .and_then(|c| c.get(1)?.as_str().parse::<u16>().ok());
    let extension_port = ext_re
        .captures_iter(content)
        .last()
        .and_then(|c| c.get(1)?.as_str().parse::<u16>().ok());

    (https_port, http_port, extension_port)
}

/// 进程匹配：忽略大小写，允许 .exe 后缀
fn collect_target_pids() -> Vec<u32> {
    let mut system = System::new();
    system.refresh_processes();

    let mut pids = system
        .processes()
        .iter()
        .filter_map(|(pid, proc_)| {
            let name = proc_.name().to_string();
            if is_target_process(&name) {
                Some(pid.as_u32())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // PID 倒序：优先扫描最新启动的渲染/子进程
    pids.sort_unstable_by(|a, b| b.cmp(a));
    pids
}

fn is_target_process(name: &str) -> bool {
    let normalized = name
        .trim()
        .to_ascii_lowercase()
        .trim_end_matches(".exe")
        .to_string();
    normalized.contains("antigravity") || normalized.contains("windsurf")
}

fn get_patterns() -> (Vec<u8>, Vec<u8>) {
    let key = "x-codeium-csrf-token";
    let pat_utf8 = key.as_bytes().to_vec();
    let pat_utf16: Vec<u8> = key.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    (pat_utf8, pat_utf16)
}

pub(crate) fn find_all_positions(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return Vec::new();
    }
    let mut positions = Vec::new();
    let mut i = 0;
    while let Some(pos) = haystack[i..].windows(needle.len()).position(|w| w == needle) {
        let absolute = i + pos;
        positions.push(absolute);
        i = absolute + 1;
        if i >= haystack.len() {
            break;
        }
    }
    positions
}

pub(crate) fn search_bytes_for_token(data: &[u8], uuid_re: &Regex, patterns: &(Vec<u8>, Vec<u8>)) -> Option<String> {
    let (pat_utf8, pat_utf16) = patterns;

    for pat in [pat_utf8, pat_utf16] {
        for pos in find_all_positions(data, pat) {
            let start = pos + pat.len();
            if start >= data.len() {
                continue;
            }
            let end = min(start + SCAN_AHEAD, data.len());
            let window = &data[start..end];

            // 尝试 UTF-8
            let utf8_text = String::from_utf8_lossy(window);
            if let Some(mat) = uuid_re.find(&utf8_text) {
                return Some(mat.as_str().to_string());
            }

            // 尝试 UTF-16LE 解码
            let utf16_units: Vec<u16> = window
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect();
            let utf16_text = String::from_utf16_lossy(&utf16_units);
            if let Some(mat) = uuid_re.find(&utf16_text) {
                return Some(mat.as_str().to_string());
            }
        }
    }

    None
}

pub fn find_csrf_token_from_memory() -> Result<String> {
    let uuid_re = Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
        .expect("valid uuid regex");
    let patterns = get_patterns();

    let pids = collect_target_pids();
    if pids.is_empty() {
        return Err(anyhow!("未找到运行中的 Antigravity/Windsurf 进程"));
    }

    for pid in pids {
        match scan_process_for_token(pid, &uuid_re, &patterns) {
            Ok(Some(token)) => return Ok(token),
            Ok(None) => continue,
            Err(e) => {
                tracing::warn!(pid, error = %e, "扫描进程失败");
                continue;
            }
        }
    }

    Err(anyhow!("未在运行中的 Antigravity/Windsurf 进程内存中找到 CSRF token"))
}
