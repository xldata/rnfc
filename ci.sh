#!/bin/bash

set -euo pipefail

export CARGO_TARGET_DIR=$PWD/target_ci
export RUSTFLAGS=-Dwarnings
export DEFMT_LOG=trace

find . -name '*.rs' -not -path '*target*' | xargs rustfmt --check  --skip-children --unstable-features --edition 2021

cargo batch  \
    --- build --release --manifest-path examples/st25r39-disco/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path examples/fm175xx/Cargo.toml --target thumbv7em-none-eabi \
