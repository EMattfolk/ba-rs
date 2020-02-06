#!/usr/bin/bash

# Kill current running bar
killall bardata
killall lemonbar

# Compile and install
cargo build --release
cp target/release/bardata ~/.local/bin/bardata

# Start the bar
./togglebar &
disown %./togglebar
