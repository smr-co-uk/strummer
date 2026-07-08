
#!/usr/bin/env bash
# Copyright 2026 smr.co.uk ltd
# SPDX-License-Identifier: Apache-2.0

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --lib --release
cargo build --release
