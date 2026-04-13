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
    let token = current_user_token(); //拿到当前用户页表的token
    let buffers =
        crate::mm::translated_byte_buffer(token, _ts as *const u8, core::mem::size_of::<TimeVal>()); //翻译用户空间的指针为内核空间的指针
    let time = crate::timer::get_time();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            &time as *const _ as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    }; //将time结构体转换为字节数组
    let mut offset = 0;
    for buffer in buffers {
        let len = buffer.len();
        buffer.copy_from_slice(&bytes[offset..offset + len]);
        offset += len;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    if _trace_request == 0 {
        return -1;
    } else if _trace_request == 1 {
        return -1;
    } else if _trace_request == 2 {
        crate::task::trace_syscall(_id);
    }
    0
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
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
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
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
