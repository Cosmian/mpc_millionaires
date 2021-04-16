#!/bin/bash

env RUST_BACKTRACE=1 RUSTC_BOOTSTRAP=1 SHARING_DATA=$(pwd)/SharingData.txt cargo build --target wasm32-unknown-unknown
