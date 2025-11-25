use anyhow::{anyhow, Result};
use mach2::kern_return::KERN_SUCCESS;
use mach2::mach_port::mach_port_name_t;
use mach2::message::mach_msg_type_number_t;
use mach2::traps::mach_task_self;
use mach2::vm::*;
use read_process_memory::{CopyAddress, Pid, ProcessHandle};
use regex::Regex;
use std::convert::TryInto;

use crate::language_server::utils::{search_bytes_for_token, CHUNK_SIZE, SCAN_AHEAD, MAX_REGION_BYTES};

pub(super) fn scan_process_for_token(
    pid: u32,
    uuid_re: &Regex,
    patterns: &(Vec<u8>, Vec<u8>),
) -> Result<Option<String>> {
    let mut task: mach_port_name_t = 0;
    let kr = unsafe { task_for_pid(mach_task_self(), pid as i32, &mut task) };
    if kr != KERN_SUCCESS {
        return Err(anyhow!("task_for_pid 失败，可能需要 sudo，kern_return={kr}"));
    }
    // 供读取的句柄（使用 read-process-memory 封装 mach_vm_read）
    let handle: ProcessHandle = (pid as Pid).try_into().map_err(|e| anyhow!("打开进程用于读取失败: {e}"))?;

    let mut address: mach_vm_address_t = 0;
    let mut size: mach_vm_size_t = 0;
    let mut info = vm_region_basic_info_data_64_t::default();
    let mut count = std::mem::size_of::<vm_region_basic_info_data_64_t>() as mach_msg_type_number_t
        / std::mem::size_of::<integer_t>() as mach_msg_type_number_t;
    let mut object_name: mach_port_name_t = 0;

    while {
        count = std::mem::size_of::<vm_region_basic_info_data_64_t>() as mach_msg_type_number_t
            / std::mem::size_of::<integer_t>() as mach_msg_type_number_t;
        unsafe {
            mach_vm_region(
                task,
                &mut address,
                &mut size,
                VM_REGION_BASIC_INFO_64,
                (&mut info as *mut vm_region_basic_info_data_64_t) as *mut _,
                &mut count,
                &mut object_name,
            )
        }
    } == KERN_SUCCESS
    {
        if info.protection & VM_PROT_READ != 0 && size > 0 {
            let mut offset: mach_vm_size_t = 0;
            let chunk: mach_vm_size_t = CHUNK_SIZE as mach_vm_size_t;
            let overlap = (patterns.0.len().max(patterns.1.len()) + SCAN_AHEAD) as mach_vm_size_t;
            let capped_size = std::cmp::min(size, MAX_REGION_BYTES as mach_vm_size_t);

            while offset < capped_size {
                let read_size = std::cmp::min(chunk, capped_size - offset);
                let mut buffer = vec![0u8; read_size as usize];
                let read_res = handle
                    .copy_address((address + offset) as usize, &mut buffer)
                    .map(|_| read_size as usize);

                let read = match read_res {
                    Ok(n) => n,
                    Err(e) => {
                        let step = read_size.saturating_sub(overlap).max(1);
                        offset = offset.saturating_add(step);
                        tracing::debug!(pid, offset, "mach_vm_read 失败: {e}");
                        continue;
                    }
                };

                buffer.truncate(read);
                if let Some(token) = search_bytes_for_token(&buffer, uuid_re, patterns) {
                    return Ok(Some(token));
                }

                let step = read_size.saturating_sub(overlap).max(1);
                offset = offset.saturating_add(step);
            }
        }
        address = address.saturating_add(size);
        if address == 0 {
            break;
        }
    }

    Ok(None)
}
