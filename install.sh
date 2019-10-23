#!/usr/bin/bash
killall bardata
killall lemonbar
cargo build --release
cp target/release/bardata ~/.local/bin/bardata
./togglebar
