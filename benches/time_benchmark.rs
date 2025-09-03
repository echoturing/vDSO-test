use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_rdtsc;

use libc::{clock_gettime, timespec, CLOCK_REALTIME, CLOCK_MONOTONIC};
use chrono::Utc;

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

/// 使用 clock_gettime 获取时间（微秒）
fn clock_gettime_us(clock: libc::clockid_t) -> u64 {
    unsafe {
        let mut ts: timespec = std::mem::zeroed();
        if clock_gettime(clock, &mut ts) != 0 {
            panic!("clock_gettime failed");
        }
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32).as_micros() as u64
    }
}

/// 使用 chrono 获取时间（微秒）
fn chrono_get_time_us() -> i64 {
    Utc::now().timestamp_micros()
}

/// Benchmark RDTSC
fn bench_rdtsc(c: &mut Criterion) {
    c.bench_function("rdtsc", |b| {
        b.iter(|| {
            black_box(rdtsc())
        })
    });
}

/// Benchmark clock_gettime with CLOCK_REALTIME
fn bench_clock_realtime(c: &mut Criterion) {
    c.bench_function("clock_gettime_realtime", |b| {
        b.iter(|| {
            black_box(clock_gettime_us(CLOCK_REALTIME))
        })
    });
}

/// Benchmark clock_gettime with CLOCK_MONOTONIC
fn bench_clock_monotonic(c: &mut Criterion) {
    c.bench_function("clock_gettime_monotonic", |b| {
        b.iter(|| {
            black_box(clock_gettime_us(CLOCK_MONOTONIC))
        })
    });
}

/// Benchmark chrono::Utc::now()
fn bench_chrono(c: &mut Criterion) {
    c.bench_function("chrono_utc_now", |b| {
        b.iter(|| {
            black_box(chrono_get_time_us())
        })
    });
}

// /// 综合比较所有时间获取方法
// fn bench_time_methods_comparison(c: &mut Criterion) {
//     let mut group = c.benchmark_group("time_methods");
    
//     // 设置测量时间和采样大小
//     group.measurement_time(Duration::from_secs(10));
//     group.sample_size(1000);
    
//     group.bench_function("rdtsc", |b| {
//         b.iter(|| black_box(rdtsc()))
//     });
    
//     group.bench_function("clock_gettime_realtime", |b| {
//         b.iter(|| black_box(clock_gettime_us(CLOCK_REALTIME)))
//     });
    
//     group.bench_function("clock_gettime_monotonic", |b| {
//         b.iter(|| black_box(clock_gettime_us(CLOCK_MONOTONIC)))
//     });
    
//     group.bench_function("chrono_utc_now", |b| {
//         b.iter(|| black_box(chrono_get_time_us()))
//     });
    
//     group.finish();
// }

// /// 测试不同调用次数下的性能表现
// fn bench_time_methods_with_iterations(c: &mut Criterion) {
//     let mut group = c.benchmark_group("time_methods_iterations");
    
//     for iterations in [1, 10, 100, 1000].iter() {
//         group.bench_with_input(
//             BenchmarkId::new("rdtsc", iterations),
//             iterations,
//             |b, &iterations| {
//                 b.iter(|| {
//                     for _ in 0..iterations {
//                         black_box(rdtsc());
//                     }
//                 })
//             },
//         );
        
//         group.bench_with_input(
//             BenchmarkId::new("clock_gettime_realtime", iterations),
//             iterations,
//             |b, &iterations| {
//                 b.iter(|| {
//                     for _ in 0..iterations {
//                         black_box(clock_gettime_us(CLOCK_REALTIME));
//                     }
//                 })
//             },
//         );
        
//         group.bench_with_input(
//             BenchmarkId::new("clock_gettime_monotonic", iterations),
//             iterations,
//             |b, &iterations| {
//                 b.iter(|| {
//                     for _ in 0..iterations {
//                         black_box(clock_gettime_us(CLOCK_MONOTONIC));
//                     }
//                 })
//             },
//         );
        
//         group.bench_with_input(
//             BenchmarkId::new("chrono_utc_now", iterations),
//             iterations,
//             |b, &iterations| {
//                 b.iter(|| {
//                     for _ in 0..iterations {
//                         black_box(chrono_get_time_us());
//                     }
//                 })
//             },
//         );
//     }
    
//     group.finish();
// }

// /// 测试连续调用的缓存效应
// fn bench_time_methods_cache_effects(c: &mut Criterion) {
//     let mut group = c.benchmark_group("time_methods_cache");
    
//     // 测试连续调用
//     group.bench_function("rdtsc_sequential", |b| {
//         b.iter(|| {
//             let _t1 = black_box(rdtsc());
//             let _t2 = black_box(rdtsc());
//             let _t3 = black_box(rdtsc());
//             let _t4 = black_box(rdtsc());
//             let _t5 = black_box(rdtsc());
//         })
//     });
    
//     group.bench_function("clock_realtime_sequential", |b| {
//         b.iter(|| {
//             let _t1 = black_box(clock_gettime_us(CLOCK_REALTIME));
//             let _t2 = black_box(clock_gettime_us(CLOCK_REALTIME));
//             let _t3 = black_box(clock_gettime_us(CLOCK_REALTIME));
//             let _t4 = black_box(clock_gettime_us(CLOCK_REALTIME));
//             let _t5 = black_box(clock_gettime_us(CLOCK_REALTIME));
//         })
//     });
    
//     group.bench_function("chrono_sequential", |b| {
//         b.iter(|| {
//             let _t1 = black_box(chrono_get_time_us());
//             let _t2 = black_box(chrono_get_time_us());
//             let _t3 = black_box(chrono_get_time_us());
//             let _t4 = black_box(chrono_get_time_us());
//             let _t5 = black_box(chrono_get_time_us());
//         })
//     });
    
//     group.finish();
// }

criterion_group!(
    benches,
    bench_rdtsc,
    bench_clock_realtime,
    bench_clock_monotonic,
    bench_chrono,
    // bench_time_methods_comparison,
    // bench_time_methods_with_iterations,
    // bench_time_methods_cache_effects
);

criterion_main!(benches);
