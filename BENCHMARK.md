# 时间获取方法性能基准测试

本项目使用 [Criterion](https://bheisler.github.io/criterion.rs/book/) 库对以下时间获取方法进行性能基准测试：

- **RDTSC**: CPU 时间戳计数器指令
- **clock_gettime(CLOCK_REALTIME)**: 系统调用获取真实时间
- **clock_gettime(CLOCK_MONOTONIC)**: 系统调用获取单调时间
- **chrono::Utc::now()**: Chrono 库的 UTC 时间获取

## 运行基准测试

### 运行所有测试
```bash
cargo bench
```

### 运行特定测试
```bash
# 只运行 RDTSC 测试
cargo bench rdtsc

# 只运行时间方法比较测试
cargo bench time_methods

# 只运行缓存效应测试
cargo bench cache
```

### 查看 HTML 报告
基准测试完成后，在 `target/criterion/` 目录下会生成详细的 HTML 报告，可以在浏览器中打开查看：

```bash
open target/criterion/index.html
```

## 测试内容

### 1. 单个方法测试
- `bench_rdtsc`: 测试 RDTSC 指令性能
- `bench_clock_realtime`: 测试 clock_gettime(CLOCK_REALTIME) 性能
- `bench_clock_monotonic`: 测试 clock_gettime(CLOCK_MONOTONIC) 性能
- `bench_chrono`: 测试 chrono::Utc::now() 性能

### 2. 方法比较测试
- `bench_time_methods_comparison`: 所有方法的综合比较
- 使用更长的测量时间（10秒）和更大的样本量（1000）

### 3. 迭代次数测试
- `bench_time_methods_with_iterations`: 测试不同调用次数（1、10、100、1000）下的性能表现
- 可以观察批量调用的性能特征

### 4. 缓存效应测试
- `bench_time_methods_cache_effects`: 测试连续调用5次的性能
- 可以观察指令缓存和数据缓存的影响

## 预期结果

一般情况下，性能排序为：
1. **RDTSC** - 最快，直接读取 CPU 寄存器
2. **clock_gettime(CLOCK_MONOTONIC)** - 通过 vDSO 优化的系统调用
3. **clock_gettime(CLOCK_REALTIME)** - 通过 vDSO 优化的系统调用
4. **chrono::Utc::now()** - 最慢，包含额外的时区处理和格式化

## 注意事项

- 测试结果会受到系统负载、CPU 频率调节等因素影响
- RDTSC 在某些虚拟化环境中可能不可用或不准确
- vDSO (Virtual Dynamic Shared Object) 的可用性会影响 clock_gettime 的性能
- 在生产环境中选择时间获取方法时，还需要考虑精度、可移植性等因素
