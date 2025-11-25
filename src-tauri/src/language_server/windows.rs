use anyhow::{anyhow, Result};
use read_process_memory::{CopyAddress, Pid, ProcessHandle};
use std::convert::TryInto;
use regex::Regex;
use std::mem::{size_of, zeroed};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Memory::{
    VirtualQueryEx, MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
    PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, PAGE_NOACCESS, PAGE_PROTECTION_FLAGS, PAGE_READONLY,
    PAGE_READWRITE, PAGE_WRITECOPY,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

use crate::language_server::utils::{search_bytes_for_token, CHUNK_SIZE, SCAN_AHEAD, MAX_REGION_BYTES};

fn is_readable(protect: PAGE_PROTECTION_FLAGS) -> bool {
    let p = protect;
    if p == PAGE_NOACCESS || p == PAGE_GUARD {
        return false;
    }
    matches!(
        p,
        PAGE_READONLY
            | PAGE_READWRITE
            | PAGE_WRITECOPY
            | PAGE_EXECUTE_READ
            | PAGE_EXECUTE_READWRITE
            | PAGE_EXECUTE_WRITECOPY
    )
}

pub(super) fn scan_process_for_token(
    pid: u32,
    uuid_re: &Regex,
    patterns: &(Vec<u8>, Vec<u8>),
) -> Result<Option<String>> {
    // 供 VirtualQueryEx 使用的句柄
    let handle: HANDLE = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)? };
    if handle.is_invalid() {
        return Err(anyhow!("OpenProcess 失败"));
    }
    // 供跨平台安全读取的句柄
    let rpm_handle: ProcessHandle = (pid as Pid).try_into().map_err(|e| anyhow!("打开进程用于读取失败: {e}"))?;

    let mut address = 0usize;
    let mut info: MEMORY_BASIC_INFORMATION = unsafe { zeroed() };

    while unsafe {
        VirtualQueryEx(
            handle,
            Some(address as *const _),
            &mut info,
            size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    } == size_of::<MEMORY_BASIC_INFORMATION>()
    {
        let region_size = info.RegionSize;
        let base = info.BaseAddress as usize;
        if info.State == MEM_COMMIT && is_readable(info.Protect) && region_size > 0 {
            let capped = std::cmp::min(region_size, MAX_REGION_BYTES);
            let overlap = patterns.0.len().max(patterns.1.len()) + SCAN_AHEAD;
            let mut offset = 0usize;

            while offset < capped {
                let chunk_size = std::cmp::min(CHUNK_SIZE, capped - offset);
                let mut buffer = vec![0u8; chunk_size];
                let read_res = rpm_handle
                    .copy_address((base + offset) as usize, &mut buffer)
                    .map(|_| chunk_size);

                let read = match read_res {
                    Ok(n) => n,
                    Err(e) => {
                        // 读失败，跳过到下一块
                        let step = std::cmp::max(1, chunk_size.saturating_sub(overlap));
                        offset = offset.saturating_add(step);
                        tracing::debug!(pid, base, offset, "ReadProcessMemory 失败: {e}");
                        continue;
                    }
                };

                buffer.truncate(read);
                if let Some(token) = search_bytes_for_token(&buffer, uuid_re, patterns) {
                    unsafe { let _ = CloseHandle(handle); };
                    return Ok(Some(token));
                }

                let step = std::cmp::max(1, read.saturating_sub(overlap));
                offset = offset.saturating_add(step);
            }
        }
        address = base.saturating_add(region_size);
        if address == 0 {
            break;
        }
    }

    unsafe { let _ = CloseHandle(handle); };
    Ok(None)
}
