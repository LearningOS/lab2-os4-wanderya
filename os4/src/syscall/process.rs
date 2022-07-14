//! Process management syscalls



use crate::config::{MAX_SYSCALL_NUM, PAGE_SIZE};
use crate::task::{
    exit_current_and_run_next,
    suspend_current_and_run_next,
    current_user_token,
    TaskStatus, get_current_status, 
    get_current_syscall_times, 
    set_mmap, 
    set_munmap
};
use crate::timer::get_time_us;
use crate::mm::{VirtAddr, MapPermission, translated_va};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// use core::mem::size_of;
// use crate::mm::copy_out;

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(ts: *mut TimeVal, tz: usize) -> isize {
    let us = get_time_us();
    // unsafe {
    //     *ts = TimeVal {
    //         sec: us / 1_000_000,
    //         usec: us % 1_000_000,
    //     };
    // }
    // let len = size_of::<TimeVal>();
    // let t = &TimeVal {
    //     sec: us / 1_000_000,
    //     usec: us % 1_000_000,
    // } as *const TimeVal as *const u8;
    // copy_out(current_user_token(), ts as usize, t, len);

    let va = VirtAddr::from(ts as usize);
    let pa = translated_va(current_user_token(), va);

    let t = pa as *mut TimeVal;
    unsafe {
        *t = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        }
    }
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    if _start % PAGE_SIZE != 0 { // 未对齐
        return -1;
    }

    if _port & !0x07 != 0 || _port & 0x07 == 0 || _len <= 0 {
        return -1;
    }

    let p = _port as u8;
    let perm = MapPermission::from_bits(p << 1).unwrap();

    let start_va: VirtAddr = VirtAddr::from(_start).floor().into();
    let end_va: VirtAddr = VirtAddr::from(_start + _len).ceil().into();

    let ok = set_mmap(start_va, end_va, perm);
    if !ok {
        return -1;
    }
    0
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if _start % PAGE_SIZE != 0 { // 未对齐
        return -1;
    }

    if _len <= 0 {
        return -1;
    }

    let  start_va: VirtAddr = VirtAddr::from(_start).floor().into();
    let end_va: VirtAddr = VirtAddr::from(_start + _len).ceil().into();
    
    let ok = set_munmap(start_va, end_va);
    if !ok {
        return -1;
    }
    0
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    // let len = size_of::<TaskInfo>();
    let time = cal_time(get_time_us());

    let va = VirtAddr::from(ti as usize);
    let pa = translated_va(current_user_token(), va);

    let ptr = pa as *mut TaskInfo;
    unsafe {
        *ptr = TaskInfo {
            status: get_current_status(),
            syscall_times: get_current_syscall_times(),
            time,
        };
    }
    // println!("sys_task_info time= {}", time);
    // tmd, 为什么TaskStatus要少一个啊
    0
}

fn cal_time(us: usize) -> usize {
    let sec= us / 1_000_000;
    let usec = us % 1_000_000;

    ((sec & 0xffff) * 1000 + usec / 1000) as usize
}