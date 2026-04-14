//! Process management syscalls
use crate::task::{
    change_program_brk, current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
};
pub const PAGE_SIZE: usize = 0x1000;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");

    if _ts.is_null() {
        return -1;
    }

    let us = crate::timer::get_time_us();
    let tv = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };

    let src = unsafe {
        core::slice::from_raw_parts(
            &tv as *const _ as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    };

    let token = current_user_token();
    let mut user_buf = crate::mm::translated_byte_buffer(token, _ts as *const u8, src.len());

    let mut copied = 0usize;
    for buf in user_buf.iter_mut() {
        let n = buf.len().min(src.len() - copied);
        buf[..n].copy_from_slice(&src[copied..copied + n]);
        copied += n;
        if copied == src.len() {
            break;
        }
    }

    if copied == src.len() {
        0
    } else {
        -1
    }
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let token = current_user_token();
    let page_table = crate::mm::PageTable::from_token(token);

    match _trace_request {
        // ===== trace_read =====
        0 => {
            let va = crate::mm::VirtAddr::from(_id);
            let vpn = va.floor();
            let offset = va.page_offset();

            if let Some(pte) = page_table.translate(vpn) {
                let flags = pte.flags();

                // 必须：有效 + 用户 + 可读
                if flags.contains(crate::mm::PTEFlags::V)
                    && flags.contains(crate::mm::PTEFlags::U)
                    && flags.contains(crate::mm::PTEFlags::R)
                {
                    let pa = pte.ppn().get_bytes_array();
                    return pa[offset] as isize;
                }
            }
            -1
        }

        // ===== trace_write =====
        1 => {
            let va = crate::mm::VirtAddr::from(_id);
            let vpn = va.floor();
            let offset = va.page_offset();

            if let Some(pte) = page_table.translate(vpn) {
                let flags = pte.flags();

                // 必须：有效 + 用户 + 可写
                if flags.contains(crate::mm::PTEFlags::V)
                    && flags.contains(crate::mm::PTEFlags::U)
                    && flags.contains(crate::mm::PTEFlags::W)
                {
                    let pa = pte.ppn().get_bytes_array();
                    pa[offset] = _data as u8;
                    return 0;
                }
            }
            -1
        }

        // ===== 查询 syscall 次数 =====
        2 => {
            if _id >= crate::config::MAX_SYSCALL_NUM {
                -1
            } else {
                crate::task::get_syscall_count(_id) as isize
            }
        }
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _start % PAGE_SIZE != 0 {
        return -1;
    }
    if _port & !0x7 != 0 {
        return -1;
    }
    if _port & 0x7 == 0 {
        return -1;
    }

    let mut perm = crate::mm::MapPermission::empty();
    if _port & 0x1 != 0 {
        perm |= crate::mm::MapPermission::R;
    }
    if _port & 0x2 != 0 {
        perm |= crate::mm::MapPermission::W;
    }
    if _port & 0x4 != 0 {
        perm |= crate::mm::MapPermission::X;
    }

    crate::task::with_current_task(|task| {
        if task.memory_set.mmap(_start, _len, perm) {
            0
        } else {
            -1
        }
    })
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if _start % PAGE_SIZE != 0 {
        return -1;
    }

    crate::task::with_current_task(|task| {
        if task.memory_set.munmap(_start, _len) {
            0
        } else {
            -1
        }
    })
}
/// change data segment sizedsadsadas
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
