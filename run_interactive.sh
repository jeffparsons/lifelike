#!/bin/sh

# For Mac OS X
export LIBRARY_PATH="$LIBRARY_PATH:/usr/local/lib"

cargo build --release

target/release/lifelike -w examples/cartesian_grid.png

