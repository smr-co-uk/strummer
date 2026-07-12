
#!/usr/bin/env bash
# Copyright 2026 smr.co.uk ltd
# SPDX-License-Identifier: Apache-2.0

cargo run -- canonical-example.strum canonical-example.mid
cargo run -- canonical-example.strum canonical-example_folk.mid --voicing folk
cargo run -- canonical-example.strum canonical-example_folk.mid --voicing rock
