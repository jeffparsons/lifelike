#!/bin/sh

cargo build --release

target/release/lifelike -w examples/cartesian_grid.png

