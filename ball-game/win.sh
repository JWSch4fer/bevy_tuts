#!/bin/sh

#better for production, smaller binary
cargo xwin build --target x86_64-pc-windows-msvc &&
cp -f target/x86_64-pc-windows-msvc/debug/ball-game.exe .
exec ./ball-game.exe "$@"
#--release this is for final production version
#cargo build --release --target x86_64-pc-windows-gnu &&

#cargo build --target x86_64-pc-windows-gnu &&
#exec ./target/x86_64-pc-windows-gnu/debug/start.exe "$@"

