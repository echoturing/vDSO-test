use libc::{clock_gettime, timespec, CLOCK_MONOTONIC, CLOCK_REALTIME};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_rdtsc;

use std::time::{Duration, Instant};

/// 获取 CPU TSC 计数值
#[cfg(target_arch = "x86_64")]
#[inline]
fn rdtsc() -> u64 {
    unsafe { _rdtsc() }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline]
fn rdtsc() -> u64 {
    // 在非 x86_64 平台上返回一个占位值
    0
}

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

fn calibrate_tsc() -> f64 {
    let start_cycles = rdtsc();
    let start = Instant::now();
    std::thread::sleep(Duration::from_millis(200)); // 200ms 测量窗口
    let elapsed = start.elapsed().as_secs_f64();
    let end_cycles = rdtsc();
    (end_cycles - start_cycles) as f64 / elapsed
}

/// 使用 TSC 获取当前时间戳（近似 CLOCK_REALTIME）
fn now_tsc(base_cycles: u64, base_time: Duration, cycles_per_sec: f64) -> Duration {
    let delta_cycles = rdtsc() - base_cycles;
    let delta_secs = delta_cycles as f64 / cycles_per_sec;
    base_time + Duration::from_secs_f64(delta_secs)
}

fn main() {
    let base_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let base_cycles = rdtsc();

    // 校准 TSC 频率
    let cycles_per_sec = calibrate_tsc();
    for i in 0..100 {
        let t1 = vdso_get_time(CLOCK_REALTIME);
        let t2 = chrono_get_time();
        let t3 = now_tsc(base_cycles, base_time, cycles_per_sec).as_micros() as i64;
        println!("{} vdso: {}, chrono: {}, tsc: {}", i, t1, t2, t3);
        std::thread::sleep(Duration::from_millis(100));
    }
}
