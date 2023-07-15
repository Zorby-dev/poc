@echo off

cargo build --bin pasm
cargo build --bin pemu
mkdir %userprofile%\.poc
copy target\debug\pasm.exe %userprofile%\.poc
copy target\debug\pemu.exe %userprofile%\.poc