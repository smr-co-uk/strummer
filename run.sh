
#!/usr/bin/env bash
# Copyright 2026 smr.co.uk ltd
# SPDX-License-Identifier: Apache-2.0

cargo run -- songs/hotel_california.strum hotel_california.mid
cargo run -- songs/hotel_california_short.strum hotel_california_short.mid
cargo run -- canonical-example.strum canonical-example.mid
cargo run -- songs/house_of_the_rising_sun.strum house_of_the_rising_sun.mid