use chrono::Utc;
use libc::{clock_gettime, timespec, CLOCK_REALTIME};
use std::time::{Duration, Instant};

fn vdso_get_time(clock: libc::clockid_t) -> u64 {
    unsafe {
        let mut ts: timespec = std::mem::zeroed();
        if clock_gettime(clock, &mut ts) != 0 {
            panic!("clock_gettime failed");
        }
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32).as_micros() as u64
    }
}

fn chrono_get_time() -> i64 {
    Utc::now().timestamp_micros()
}

fn main() {
    // 预热
    for _ in 0..10000 {
        let _ = vdso_get_time(CLOCK_REALTIME);
        let _ = chrono_get_time();
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
        let _ = chrono_get_time();
        count += 1;
    }
    let chrono_qps = count as f64 / start.elapsed().as_secs_f64();

    println!("vdso_qps: {:.0} QPS", vdso_qps);
    println!("chrono_qps:  {:.0} QPS", chrono_qps);
    println!(
        "  性能差异: {:.2}%",
        (vdso_qps - chrono_qps) / chrono_qps * 100.0
    );
}
