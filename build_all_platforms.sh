#!/bin/bash
set -euo pipefail
SECONDS=0
echo "this script is meant to be used to compile for github releases"
echo "this script requires zip"
echo "cleaning workspace..."
cargo clean
echo "creating output folder"
mkdir -p compressed_out
echo "beginning compilation"
echo "==== compiling for windows ===="
echo "building for x86_64 windows"
cargo build --target x86_64-pc-windows-gnu --release
echo "building for i686 windows"
cargo build --target i686-pc-windows-gnu --release
echo "building for aarch64 windows"
cargo build --target aarch64-pc-windows-gnullvm --release
echo "==== compiling for linux ===="
echo "statically building for x86_64 linux"
cargo build --target x86_64-unknown-linux-musl --release
echo "statically building for i686 linux"
cargo build --target i686-unknown-linux-musl --release
echo "statically building for aarch64 linux"
cargo build --target aarch64-unknown-linux-musl --release
echo "==== compiling for macOS ===="
echo "building for aarch64 apple darwin"
cargo build --target aarch64-apple-darwin --release
echo "building complete, packaging executables"
echo "==== packaging for windows"
echo "packaging x86_64 windows"
mkdir target/x86_64-pc-windows-gnu/win-x86_64
mv target/x86_64-pc-windows-gnu/client.exe target/x86_64-pc-windows-gnu/server.exe target/x86_64-pc-windows-gnu/win-x86_64
zip -r target/x86_64-pc-windows-gnu/win-x86_64
mv target/x86_64-pc-windows-gnu/win-x86_64.zip compressed_out/
echo "packaging i686 windows"
mkdir target/i686-pc-windows-gnu/win-i686
mv target/i686-pc-windows-gnu/client.exe target/i686-pc-windows-gnu/server.exe target/i686-pc-windows-gnu/win-i686
zip -r target/i686-pc-windows-gnu/win-i686
mv target/i686-pc-windows-gnu/win-i686.zip compressed_out/
echo "packaging aarch64 windows"
mkdir target/aarch64-pc-windows-gnu/win-aarch64
mv target/aarch64-pc-windows-gnu/client.exe target/aarch64-pc-windows-gnu/server.exe target/aarch64-pc-windows-gnu/win-aarch64
zip -r target/aarch64-pc-windows-gnu/win-aarch64
mv target/aarch64-pc-windows-gnu/win-aarch64.zip compressed_out/
echo "==== packaging for linux ===="
echo "packaging for x86_64 linux"
mkdir target/x86_64-unknown-linux-musl/linux-x86_64
mv target/x86_64-unknown-linux-musl/client target/x86_64-unknown-linux-musl/server target/x86_64-unknown-linux-musl/linux-x86_64
zip -r target/x86_64-unknown-linux-musl/linux-x86_64
mv target/x86_64-unknown-linux-musl/linux-x86_64.zip compressed_out/
echo "packaging for i686 linux"
mkdir target/i686-unknown-linux-musl/linux-i686
mv target/i686-unknown-linux-musl/client target/i686-unknown-linux-musl/server target/i686-unknown-linux-musl/linux-i686
zip -r target/i686-unknown-linux-musl/linux-i686
mv target/i686-unknown-linux-musl/linux-i686.zip compressed_out/
echo "packaging for aarch64 linux"
mkdir target/aarch64-unknown-linux-musl/linux-aarch64
mv target/aarch64-unknown-linux-musl/client target/aarch64-unknown-linux-musl/server target/aarch64-unknown-linux-musl/linux-aarch64
zip -r target/aarch64-unknown-linux-musl/linux-aarch64
mv target/aarch64-unknown-linux-musl/linux-aarch64.zip compressed_out/
echo "==== packaging for macOS ===="
mkdir target/aarch64-apple-darwin/darwin-aarch64
mv target/aarch64-apple-darwin/client target/aarch64-apple-darwin/server target/aarch64-apple-darwin/darwin-aarch64
zip -r target/aarch64-apple-darwin/darwin-aarch64
mv target/aarch64-apple-darwin/darwin-aarch64.zip compressed_out/
echo "==== FINISHED ===="
echo "gui is not built because every platform requires different libraries"
echo "building and packaging took ${SECONDS} seconds"