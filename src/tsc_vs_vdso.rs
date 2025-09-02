use std::arch::x86_64::_rdtsc;
use std::time::{Duration, Instant};

/// 获取 CPU TSC 计数值
#[inline]
fn rdtsc() -> u64 {
    unsafe { _rdtsc() }
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

    // 模拟多次获取时间戳
    for _ in 0..5 {
        let ts = now_tsc(base_cycles, base_time, cycles_per_sec);
        println!("ts = {:?}", ts);
        std::thread::sleep(Duration::from_millis(500));
    }
}