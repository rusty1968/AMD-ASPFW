#!/bin/bash

# Build script for bignum-rs FFI demonstration
# Builds the Rust library and links it with C test program

set -e

echo "=== Building BigNum-RS FFI Demo ==="

# Build the Rust library as a static library
echo "Building Rust library..."
cargo build --release

# Create symlink to the static library with expected name
ln -sf target/release/libbignum_rs.a target/release/libbignum.a

# Compile the C test program
echo "Compiling C test program..."
gcc -Wall -Wextra -std=c99 \
    -I include \
    -L target/release \
    -o target/release/test_ffi \
    examples/test_bignum_ffi.c \
    -lbignum_rs \
    -lpthread -ldl -lm

echo "Build complete!"
echo ""
echo "To run the test:"
echo "  ./target/release/test_ffi"
echo ""
echo "To run Rust tests:"
echo "  cargo test"
