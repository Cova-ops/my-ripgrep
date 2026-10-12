# RustGrep

A small educational project that implements a concurrent text search
tool inspired by `ripgrep`.

> **Disclaimer**
>
> This project was created **purely for learning purposes**. The goal
> was to practice Rust fundamentals such as ownership, borrowing,
> concurrency, channels, synchronization, file I/O, error handling, and
> project organization---not to replace or compete with `ripgrep`.

## Features

- Recursive directory traversal
- Concurrent file processing using a custom thread pool
- Streaming file reading with `BufReader`
- Case-insensitive search
- File extension filtering
- Directory exclusion
- Summary statistics
- Cross-platform (Linux, macOS, Windows)

## Why this project?

Instead of relying on existing crates for concurrency, the project
implements a custom thread pool using Rust's standard library to better
understand:

- `Arc`
- `Mutex`
- `mpsc` channels
- `JoinHandle`
- Ownership across threads
- Message-passing concurrency

Several performance bottlenecks were identified and optimized during
development, including:

- Filtering files before scheduling work
- Eliminating unnecessary allocations inside hot loops
- Reusing buffers while reading files
- Removing unnecessary synchronization
- Reducing filesystem overhead

Each optimization was benchmarked incrementally to measure its impact.

## Benchmark

Benchmarking was performed using `hyperfine` against `ripgrep`.

The dataset consisted of approximately **10 real-world TypeScript
repositories**.

Both implementations produced the **same search results (543 matching
lines)** for the benchmark query.

Example benchmark:

```text
RustGrep : 45.6 ms
ripgrep  : 46.8 ms
```

This should **not** be interpreted as RustGrep being generally faster
than `ripgrep`.

`ripgrep` is a production-grade tool with years of engineering effort
and many advanced features (regular expressions, ignore rules, Unicode
support, SIMD optimizations, highly optimized filesystem traversal, and
much more).

The benchmark simply demonstrates that, for one specific literal search
on one dataset, this educational implementation achieved comparable
performance.

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release -- <pattern> <directory> [options]
```

Example:

```bash
cargo run --release -- Video ./projects \
    --ignore-case \
    --exclude .git \
    --exclude node_modules \
    --extension ts
```

## License

This repository is intended for educational purposes.
