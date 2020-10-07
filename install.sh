#!/usr/bin/bash

# Compile
cargo build --release
status=$?
if [[ "$status" != "0" ]]; then
    echo "Aborting compilation"
    exit $status
fi

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
