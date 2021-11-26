# qoi_rs

## What is this?

A pretty boring Rust translation of [qoi](https://github.com/phoboslab/qoi).

## Status

### What's there

- Encode & Decode works
- Results agree with the C implementation for all [samples images](https://phoboslab.org/files/qoibench/images.tar).
- No unsafe code (and in fact `#![forbid(unsafe_code)]`)
- No dependencies
- `#[no_std]` compatible (but you need `alloc`).

### What's not yet there

- Benchmarks
- A CLI tool
- A C API