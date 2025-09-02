use libc::{clock_gettime, timespec, CLOCK_MONOTONIC, CLOCK_REALTIME};
use std::time::Duration;

fn vdso_get_time(clock: libc::clockid_t) -> i64 {
    unsafe {
        let mut ts: timespec = std::mem::zeroed();
        if clock_gettime(clock, &mut ts) != 0 {
            panic!("clock_gettime failed");
        }
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32).as_micros() as i64
    }
}
fn chrono_get_time() -> i64 {
    chrono::Utc::now().timestamp_micros()
}

fn main() {
    // 启动时记录基准

    // 模拟业务获取“真实时间”
    for _ in 0..100000 {
        let vdso_ts = vdso_get_time(CLOCK_REALTIME);
        let chrono_ts = chrono_get_time();
        println!(" {:?}", chrono_ts - vdso_ts);
        println!(" \n");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
