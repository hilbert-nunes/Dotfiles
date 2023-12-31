#!/bin/sh

set -ex

export CARGO_NET_RETRY=5
export CARGO_NET_TIMEOUT=10

MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri)
echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
rustup default "$MIRI_NIGHTLY"

rustup component add miri
cargo miri setup

# Disable isolation for num_cpus::get_physical.
MIRIFLAGS="-Zmiri-disable-isolation" \
MMTEST_FAST_TEST=1 \
    cargo miri test "$@"
