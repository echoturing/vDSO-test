use std::arch::x86_64::_rdtsc;
use libc::{clock_gettime, timespec, CLOCK_REALTIME};
use std::time::{Duration, Instant};

/// 获取 CPU TSC 计数值
#[inline]
fn rdtsc() -> u64 {
    unsafe { _rdtsc() }
}

fn vdso_get_time(clock: libc::clockid_t) -> u64 {
    unsafe {
        let mut ts: timespec = std::mem::zeroed();
        if clock_gettime(clock, &mut ts) != 0 {
            panic!("clock_gettime failed");
        }
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32).as_micros() as u64
    }
}

/// 校准: 测量每秒钟增加多少 tick
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
    // 启动时用 CLOCK_REALTIME 做基准
    let base_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let base_cycles = rdtsc();

    // 校准 TSC 频率
    let cycles_per_sec = calibrate_tsc();
    println!("TSC frequency ~ {:.3} GHz", cycles_per_sec / 1e9);

    // 预热
    for _ in 0..10000 {
        let _ = vdso_get_time(CLOCK_REALTIME);
        let _ = now_tsc(base_cycles, base_time, cycles_per_sec);
    }

    // 测试 CLOCK_REALTIME
    let start = Instant::now();
    let mut count = 0;
    while start.elapsed().as_secs() < 5 {
        let _ = vdso_get_time(CLOCK_REALTIME);
        count += 1;
    }
    let vdso_qps = count as f64 / start.elapsed().as_secs_f64();

    // 测试 CLOCK_REALTIME
    let start = Instant::now();
    let mut count = 0;
    while start.elapsed().as_secs() < 5 {
        let _ = now_tsc(base_cycles, base_time, cycles_per_sec);
        count += 1;
    }
    let tsc_qps = count as f64 / start.elapsed().as_secs_f64();

    println!("vdso_qps: {:.0} QPS", vdso_qps);
    println!("tsc_qps:  {:.0} QPS", tsc_qps);
    println!(
        "  性能差异: {:.2}%",
        (vdso_qps - tsc_qps) / tsc_qps * 100.0
    );
}
