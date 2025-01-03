# wring - *double uring*

[![Crates.io](https://img.shields.io/crates/v/wring)](https://crates.io/crates/wring)
[![Docs.rs](https://docs.rs/wring/badge.svg)](https://docs.rs/wring)
[![License](https://img.shields.io/crates/l/wring)](https://crates.io/crates/wring)

Rust `io_uring` library compatible with Tokio.

> [!WARNING]  
> This library is under development. Contributions are welcome.

## Objective

This library is mainly used to speed up async file I/O operations. Turning this into a full-fledged async runtime is not the immediate main goal.

## Usage

```rs
use wring::fs::File;

#[tokio::main]
async fn main() {
    let mut _guard = wring::background(64).unwrap();
    let mut buf = Vec::with_capacity(1024);

    let file = File::open("Cargo.toml").await.unwrap();
    file.read(&mut buf, 0).await.unwrap();

    let text = String::from_utf8(buf).unwrap();
    println!("{}", text);
}
```

## Roadmap

- [x] Basic File I/O
- [x] Background Runner (as Double)
- [ ] Async Runtime
- [ ] Network I/O
- [ ] Timer
- [ ] Tests
- [ ] Benchmarks
- [ ] Documentation
- [ ] Examples
- [ ] CI/CD
