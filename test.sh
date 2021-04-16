#!/bin/bash

env RUST_BACKTRACE=1 RUSTC_BOOTSTRAP=1 RUST_TEST_THREADS=1 \
cargo test --features emulate --bin scale_wasm --target x86_64-unknown-linux-gnu -- tests --nocapture