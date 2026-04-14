//! Process management syscalls
use crate::task::{
    change_program_brk, current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
};
use alloc::vec::Vec;

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

    let token: usize = current_user_token();

    let us: usize = crate::timer::get_time_us();
    let tv: TimeVal = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };

    let size: usize = core::mem::size_of::<TimeVal>();

    let buffers: Vec<&mut [u8]> = crate::mm::translated_byte_buffer(token, _ts as *const u8, size);

    let total_len: usize = buffers.iter().map(|b| b.len()).sum();
    if total_len < size {
        return -1;
    }

    let src: &[u8] = unsafe { core::slice::from_raw_parts(&tv as *const _ as *const u8, size) };

    let mut offset: usize = 0;
    for buf in buffers {
        let len = buf.len();
        buf.copy_from_slice(&src[offset..offset + len]);
        offset += len;
    }

    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
use crate::mm::{PageTable, PageTableEntry, PTEFlags, VirtAddr};

pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");

    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    let va = VirtAddr::from(_id);
    let vpn = va.floor();
    let offset = va.page_offset();

    match _trace_request {
        0 => {
            let pte: PageTableEntry = match page_table.translate(vpn) {
                Some(pte) => pte,
                None => return -1,
            };
            let flags = pte.flags();
            if !pte.is_valid() || !pte.readable() || !flags.contains(PTEFlags::U) {
                return -1;
            }
            pte.ppn().get_bytes_array()[offset] as isize
        }
        1 => {
            let pte: PageTableEntry = match page_table.translate(vpn) {
                Some(pte) => pte,
                None => return -1,
            };
            let flags = pte.flags();
            if !pte.is_valid() || !pte.writable() || !flags.contains(PTEFlags::U) {
                return -1;
            }
            pte.ppn().get_bytes_array()[offset] = (_data & 0xff) as u8;
            0
        }
        2 => crate::task::get_syscall_count(_id) as isize,
        _ => -1,
    }
}



// YOUR JOB: Implement mmap.
use core::arch::asm;

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    let ret = crate::task::TASK_MANAGER.with_current_task(|task| {
        task.memory_set.mmap(start, len, prot)
    });
    if ret == 0 {
        unsafe { asm!("sfence.vma"); }
    }
    ret
}
/// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let ret = crate::task::TASK_MANAGER.with_current_task(|task| {
        task.memory_set.munmap(start, len)
    });
    if ret == 0 {
        unsafe { asm!("sfence.vma"); }
    }
    ret
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
