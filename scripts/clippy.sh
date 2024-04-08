#!/bin/bash

cargo clippy "$@" --all-targets --all-features -- -D warnings -D future-incompatible -D nonstandard-style -D rust-2018-idioms -D unused -D clippy::unwrap_used -A clippy::blocks_in_conditions