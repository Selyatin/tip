#!/bin/sh
cargo build --release --target x86_64-pc-windows-gnu
echo "--- Windows Build Finished ---"
cargo build --release --target x86_64-unknown-linux-musl
echo "--- Linux x64 Build Finished ---"
cargo build --release --target aarch64-unknown-linux-musl
echo "--- Linux aarch64 Build Finished ---"
cargo build --release --target x86_64-apple-darwin
echo "--- Darwin x64 Build Finished ---"
cargo build --release --target aarch64-apple-darwin
echo "--- Darwin aarch64 Build Finished ---"
