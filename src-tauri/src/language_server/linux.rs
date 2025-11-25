use anyhow::{anyhow, Context, Result};
use read_process_memory::{CopyAddress, Pid, ProcessHandle};
use regex::Regex;
use std::fs;
use std::convert::TryInto;

use crate::language_server::utils::{search_bytes_for_token, CHUNK_SIZE, SCAN_AHEAD, MAX_REGION_BYTES};

#[derive(Debug)]
struct Region {
    start: u64,
    end: u64,
}

pub(super) fn scan_process_for_token(
    pid: u32,
    uuid_re: &Regex,
    patterns: &(Vec<u8>, Vec<u8>),
) -> Result<Option<String>> {
    let maps_path = format!("/proc/{pid}/maps");
    let maps = fs::read_to_string(&maps_path).with_context(|| format!("读取 {maps_path} 失败"))?;

    let mut regions = Vec::new();
    for line in maps.lines() {
        let mut parts = line.split_whitespace();
        if let Some(range) = parts.next() {
            if let Some(perms) = parts.next() {
                if !perms.contains('r') {
                    continue;
                }
                let mut segs = range.split('-');
                if let (Some(s), Some(e)) = (segs.next(), segs.next()) {
                    if let (Ok(start), Ok(end)) = (u64::from_str_radix(s, 16), u64::from_str_radix(e, 16)) {
                        if end > start {
                            regions.push(Region { start, end });
                        }
                    }
                }
            }
        }
    }

    let handle: ProcessHandle = (pid as Pid).try_into().map_err(|e| anyhow!("打开进程用于读取失败: {e}"))?;

    let overlap = patterns.0.len().max(patterns.1.len()) + SCAN_AHEAD;

    for region in regions {
        let mut cursor = region.start;
        let region_cap_end = (region.start + (MAX_REGION_BYTES as u64)).min(region.end);
        while cursor < region_cap_end {
            let remaining = (region_cap_end - cursor) as usize;
            if remaining == 0 {
                break;
            }
            let chunk_size = remaining.min(CHUNK_SIZE);

            let mut buffer = vec![0u8; chunk_size];
            let read_res = handle
                .copy_address(cursor as usize, &mut buffer)
                .map(|_| chunk_size);

            let read = match read_res {
                Ok(n) => n,
                Err(e) => {
                    let step = chunk_size.saturating_sub(overlap).max(1) as u64;
                    cursor = cursor.saturating_add(step);
                    tracing::debug!(pid, cursor, "读取 0x{:x} 失败: {e}", cursor);
                    continue;
                }
            };

            buffer.truncate(read);
            if let Some(token) = search_bytes_for_token(&buffer, uuid_re, patterns) {
                return Ok(Some(token));
            }

            let step = read.saturating_sub(overlap).max(1) as u64;
            cursor = cursor.saturating_add(step);
        }
    }

    Ok(None)
}
