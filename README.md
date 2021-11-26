# qoi_rs

[![crates.io](https://img.shields.io/crates/v/qoi_rs.svg)](https://crates.io/crates/qoi_rs)
[![Documentation](https://docs.rs/qoi_rs/badge.svg)](https://docs.rs/qoi_rs)
[![MIT licensed](https://img.shields.io/crates/l/qoi_rs.svg)](./LICENSE)
[![CI](https://github.com/whentze/qoi_rs/actions/workflows/rust.yml/badge.svg)](https://github.com/whentze/qoi_rs/actions)


## What is this?

A pretty boring Rust translation of [qoi](https://github.com/phoboslab/qoi).

## Status

### What's there

- Encode & Decode works
- Results agree with the C implementation for all [samples images](https://phoboslab.org/files/qoibench/images.tar).
- No unsafe code (and in fact `#![forbid(unsafe_code)]`)
- No dependencies
- `#[no_std]` compatible (but you need `alloc`).
- Not a lot of code

### What's not yet there

- `io::{Read/Write}` style functions
- API Docs
- Benchmarks
- A CLI tool
- A C API