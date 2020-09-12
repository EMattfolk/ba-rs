#!/usr/bin/bash

# Compile
cargo build --release

# Kill current running bar
killall bardata
killall lemonbar

mkdir -p ~/.local/bin

# Copy over the bar
cp target/release/bardata ~/.local/bin/bardata
cp togglebar ~/.local/bin/togglebar

# Start the bar
./togglebar &
disown %./togglebar
