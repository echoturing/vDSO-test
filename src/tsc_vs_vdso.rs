use libc::{clock_gettime, timespec, CLOCK_REALTIME};

// Cargo.toml 里建议添加：
// [features]
// tsc = []   # 仅在 linux+x86_64 有效

use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
struct Base {
    wall_base: Duration,
    mono_base: Duration,
}

static BASE: OnceLock<Base> = OnceLock::new();

#[inline]
pub fn now_wall() -> Duration {
    let base = *BASE.get_or_init(init_base);
    base.wall_base + (now_mono() - base.mono_base)
}

#[inline]
pub fn now_mono() -> Duration {
    platform_now_mono()
}

// ---------- 初始化：启动时记录一次 wall & mono ----------
fn init_base() -> Base {
    let wall = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let mono = platform_now_mono();
    Base {
        wall_base: wall,
        mono_base: mono,
    }
}

// ==========================================================
// 平台实现：单调时间（尽量走最快路径）
// ==========================================================

// ---------- Linux ----------
#[cfg(target_os = "linux")]
fn platform_now_mono() -> Duration {
    // 优先：在 linux+x86_64 且启用 feature "tsc" 时，走 RDTSC 极致路径
    #[cfg(all(target_arch = "x86_64", feature = "tsc"))]
    {
        return tsc_now_mono();
    }
    // 其他情况：走 vDSO 的 clock_gettime(CLOCK_MONOTONIC)
    vdso_clock_gettime_monotonic()
}

#[cfg(target_os = "linux")]
#[inline]
fn vdso_clock_gettime_monotonic() -> Duration {
    use libc::{clock_gettime, timespec, CLOCK_MONOTONIC};
    unsafe {
        let mut ts: timespec = std::mem::zeroed();
        let ret = clock_gettime(CLOCK_MONOTONIC, &mut ts as *mut _);
        if ret != 0 {
            // 极少数情况下出错，退回 0
            return Duration::from_nanos(0);
        }
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
    }
}

// ---------- Linux x86_64: TSC 超快路径（可选） ----------
#[cfg(all(target_os = "linux", target_arch = "x86_64", feature = "tsc"))]
fn tsc_now_mono() -> Duration {
    // 用 OnceLock 做一次频率校准与基准记录
    struct TscCalib {
        cycles_per_sec: f64,
        base_cycles: u64,
        base_mono: Duration, // 与 TSC 对齐的单调时间基准
    }
    static CALIB: OnceLock<TscCalib> = OnceLock::new();

    let c = CALIB.get_or_init(|| {
        // 以 vDSO 的 MONOTONIC 为“真值”，校准 TSC 频率
        let start_cycles = unsafe { core::arch::x86_64::_rdtsc() };
        let start_mono = vdso_clock_gettime_monotonic();
        // 采用较短睡眠窗口，权衡启动延迟与精度
        std::thread::sleep(Duration::from_millis(50));
        let end_cycles = unsafe { core::arch::x86_64::_rdtsc() };
        let end_mono = vdso_clock_gettime_monotonic();

        let d_cycles = (end_cycles - start_cycles) as f64;
        let d_secs = (end_mono - start_mono).as_secs_f64();
        let cps = if d_secs > 0.0 { d_cycles / d_secs } else { 0.0 };

        TscCalib {
            cycles_per_sec: cps.max(1.0), // 防守式，避免被 0 除
            base_cycles: end_cycles,
            base_mono: end_mono,
        }
    });

    let now_cycles = unsafe { core::arch::x86_64::_rdtsc() };
    let delta_cycles = now_cycles.wrapping_sub(c.base_cycles) as f64;
    let delta_secs = delta_cycles / c.cycles_per_sec;
    c.base_mono + Duration::from_secs_f64(delta_secs.max(0.0))
}

// ---------- macOS（Intel 与 Apple Silicon 通用） ----------
#[cfg(target_os = "macos")]
fn platform_now_mono() -> Duration {
    // 使用 mach_absolute_time（极快的单调计时器）
    // ticks -> ns 需要 timebase (numer/denom)
    use core::mem::MaybeUninit;

    #[repr(C)]
    struct mach_timebase_info_data_t {
        numer: u32,
        denom: u32,
    }

    extern "C" {
        fn mach_absolute_time() -> u64;
        fn mach_timebase_info(info: *mut mach_timebase_info_data_t) -> i32;
    }

    // 获取并缓存 timebase
    static TBASE_NUM: OnceLock<u64> = OnceLock::new();
    static TBASE_DEN: OnceLock<u64> = OnceLock::new();

    let (numer, denom) = {
        let n = TBASE_NUM.get();
        let d = TBASE_DEN.get();
        if let (Some(&n), Some(&d)) = (n, d) {
            (n, d)
        } else {
            // 首次初始化
            let mut info = MaybeUninit::<mach_timebase_info_data_t>::uninit();
            let kr = unsafe { mach_timebase_info(info.as_mut_ptr()) };
            if kr != 0 {
                return Duration::from_nanos(0);
            }
            let info = unsafe { info.assume_init() };
            let n = info.numer as u64;
            let d = info.denom as u64;
            let _ = TBASE_NUM.set(n);
            let _ = TBASE_DEN.set(d);
            (n, d)
        }
    };

    let t = unsafe { mach_absolute_time() }; // 原始 ticks
                                             // 转纳秒：ticks * numer / denom
    let nanos = if denom != 0 {
        // 用 u128 避免溢出
        ((t as u128) * (numer as u128) / (denom as u128)) as u64
    } else {
        0
    };
    Duration::from_nanos(nanos)
}

// ---------- 其他平台（保底：使用 std 的 Instant 差值） ----------
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn platform_now_mono() -> Duration {
    // 兼容路径：Instant 相对单调，转成启动以来的 Duration
    static START: OnceLock<std::time::Instant> = OnceLock::new();
    let s = *START.get_or_init(std::time::Instant::now);
    s.elapsed()
}

// ---------------------- 示例 ----------------------
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smoke() {
        let a = now_wall();
        std::thread::sleep(Duration::from_millis(10));
        let b = now_wall();
        assert!(b > a);
    }
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

fn main() {
    for _ in 0..10000 {
        let _ = vdso_get_time(CLOCK_REALTIME);
        let _ = now_wall();
    }

    // 测试 CLOCK_REALTIME
    let start = Instant::now();
    let mut count = 0;
    while start.elapsed().as_secs() < 5 {
        let _ = vdso_get_time(CLOCK_REALTIME);
        count += 1;
    }
    let vdso_qps = count as f64 / start.elapsed().as_secs_f64();

    // 测试 tsc
    let start = Instant::now();
    let mut count = 0;
    while start.elapsed().as_secs() < 5 {
        let _ = now_wall();
        count += 1;
    }
    let tsc_qps = count as f64 / start.elapsed().as_secs_f64();

    println!("vdso_qps: {:.0} QPS", vdso_qps);
    println!("chrono_qps:  {:.0} QPS", tsc_qps);
    println!("  性能差异: {:.2}%", (vdso_qps - tsc_qps) / tsc_qps * 100.0);
}
