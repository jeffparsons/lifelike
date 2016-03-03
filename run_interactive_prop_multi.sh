#!/bin/sh

# For Mac OS X
export LIBRARY_PATH="$LIBRARY_PATH:/usr/local/lib"

cargo build --release

target/release/lifelike -wp --smin=2 --smax=6 --rmin=5 --rmax=6 examples/hex_square_tri_large.png

