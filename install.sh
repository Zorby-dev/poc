#!/bin/sh
cargo build --bin pasm
cargo build --bin pemu
mv target/debug/pasm ~/bin
mv target/debug/pemu ~/bin