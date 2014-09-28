#!/bin/sh

cargo build --release

target/release/lifelike -f200 -w -o square examples/cartesian_grid.png

target/release/lifelike -f200 -w -o hex --smin=2 --smax=4 --rmin=3 --rmax=4 examples/hex_grid.png
target/release/lifelike -f200 -w -o alt_hex --smin=1 --smax=3 --rmin=3 --rmax=4 examples/hex_grid.png
target/release/lifelike -f200 -w -o stabilize --smin=2 --smax=3 --rmin=2 --rmax=2 examples/hex_grid.png

target/release/lifelike -f200 -w -o multi --smin=3 --smax=4 --rmin=3 --rmax=4 examples/hex_square_tri_large.png

target/release/lifelike -f300 -wp -o prop_multi --smin=2 --smax=6 --rmin=5 --rmax=6 examples/hex_square_tri_large.png
