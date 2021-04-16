#!/bin/bash

env RUST_BACKTRACE=1 RUSTC_BOOTSTRAP=1 \
cargo run --features emulate --target x86_64-unknown-linux-gnu -- --no-capture