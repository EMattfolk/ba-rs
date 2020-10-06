#!/usr/bin/bash

# Compile
cargo build --release

# Kill current running bar
killall ba
killall lemonbar

mkdir -p ~/.local/bin

# Copy over the bar
cp target/release/ba ~/.local/bin/ba
cp togglebar ~/.local/bin/togglebar

# Start the bar
./togglebar &
disown %./togglebar
