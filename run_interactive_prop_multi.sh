#!/bin/sh

cargo build --release

target/release/lifelike -wp --smin=2 --smax=6 --rmin=5 --rmax=6 examples/hex_square_tri_large.png

