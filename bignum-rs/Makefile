# BigNum-RS Build System with Unit Tests and Coverage
# This builds the Rust library, C unit tests, and FFI compatibility tests

# Project configuration
PROJECT_NAME = bignum_rs
RUST_LIB_NAME = lib$(PROJECT_NAME).a

# Directories
SRC_DIR = src
EXAMPLES_DIR = examples
INCLUDE_DIR = include
TEST_DIR = tests
TARGET_DIR = target/release
TARGET_DEBUG_DIR = target/debug
BUILD_DIR = build

# Compiler and flags
CC = gcc
CFLAGS = -Wall -Wextra -O2 -I$(INCLUDE_DIR)
CFLAGS_COVERAGE = -Wall -Wextra -O0 -g --coverage -fprofile-arcs -ftest-coverage -I$(INCLUDE_DIR)
LIBS = -lpthread -ldl -lm
LIBS_COVERAGE = -lpthread -ldl -lm -lgcov

# Rust configuration
CARGO = cargo
CARGO_FLAGS = --release

# Test targets
UNIT_TEST = $(BUILD_DIR)/unit_test
FFI_TEST = $(BUILD_DIR)/test_bignum_ffi
COVERAGE_TEST = $(BUILD_DIR)/unit_test_coverage

.PHONY: all clean test unit-test coverage rust-lib debug-lib clippy help install info

# Default target
all: rust-lib unit-test

help:
	@echo "=== BigNum-RS Build System ==="
	@echo ""
	@echo "Available targets:"
	@echo "  all          - Build library and run unit tests"
	@echo "  rust-lib     - Build Rust library (release)"
	@echo "  debug-lib    - Build Rust library (debug)"
	@echo "  unit-test    - Build and run focused unit tests"
	@echo "  coverage     - Build and run tests with coverage analysis"
	@echo "  ffi-test     - Build and run comprehensive FFI tests"
	@echo "  clippy       - Run Rust clippy linter for code quality"
	@echo "  lint         - Alias for clippy"
	@echo "  memcheck     - Run unit tests with valgrind (if available)"
	@echo "  clean        - Clean all build artifacts"
	@echo "  install      - Install library system-wide"
	@echo "  info         - Show detailed project information"
	@echo ""
	@echo "Development targets:"
	@echo "  quick        - Run unit tests only (fast)"
	@echo "  full         - Run all tests + coverage (comprehensive)"
	@echo "  dev          - Build debug + unit tests"
	@echo "  ci           - Full CI pipeline (clippy + all tests + coverage)"
	@echo ""

# Build Rust library (release)
rust-lib: $(TARGET_DIR)/$(RUST_LIB_NAME)

$(TARGET_DIR)/$(RUST_LIB_NAME):
	@echo "Building Rust library (release)..."
	$(CARGO) build $(CARGO_FLAGS)
	@echo "✓ Rust library built successfully"

# Build Rust library (debug)
debug-lib: $(TARGET_DEBUG_DIR)/$(RUST_LIB_NAME)

$(TARGET_DEBUG_DIR)/$(RUST_LIB_NAME):
	@echo "Building Rust library (debug)..."
	$(CARGO) build
	@echo "✓ Rust library (debug) built successfully"

# Run Rust clippy linter
clippy:
	@echo ""
	@echo "=== Running Clippy Linter ==="
	@echo "Checking code quality and style..."
	$(CARGO) clippy --all-targets --all-features -- -D warnings
	@echo "✓ Clippy linting completed successfully"
	@echo ""

# Alias for clippy
lint: clippy

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Build and run unit tests
unit-test: $(UNIT_TEST)
	@echo ""
	@echo "=== Running Unit Tests ==="
	./$(UNIT_TEST)
	@echo ""

$(UNIT_TEST): $(TARGET_DIR)/$(RUST_LIB_NAME) $(BUILD_DIR)
	@echo "Building unit test program..."
	$(CC) $(CFLAGS) -o $(UNIT_TEST) \
		$(TEST_DIR)/unit_test.c \
		$(TARGET_DIR)/$(RUST_LIB_NAME) \
		$(LIBS)
	@echo "✓ Unit test program built"

# Build and run coverage analysis
coverage: $(COVERAGE_TEST)
	@echo ""
	@echo "=== Running Coverage Analysis ==="
	./$(COVERAGE_TEST)
	@echo ""
	@echo "=== Generating Coverage Report ==="
	@cd $(TEST_DIR) && gcov -r unit_test.c
	@echo ""
	@echo "=== Coverage Summary ==="
	@cd $(TEST_DIR) && gcov -n unit_test.c | grep -E "(File|Lines executed|Creating)" || true
	@echo ""
	@echo "Coverage files generated in $(TEST_DIR)/ directory"
	@ls -la $(TEST_DIR)/*.gcov 2>/dev/null || echo "No .gcov files found"

$(COVERAGE_TEST): $(TARGET_DIR)/$(RUST_LIB_NAME) $(BUILD_DIR)
	@echo "Building unit test program with coverage..."
	$(CC) $(CFLAGS_COVERAGE) -o $(COVERAGE_TEST) \
		$(TEST_DIR)/unit_test.c \
		$(TARGET_DIR)/$(RUST_LIB_NAME) \
		$(LIBS_COVERAGE)
	@echo "✓ Coverage test program built"

# Build and run comprehensive FFI tests
ffi-test: $(FFI_TEST)
	@echo ""
	@echo "=== Running Comprehensive FFI Tests ==="
	./$(FFI_TEST)
	@echo ""

$(FFI_TEST): $(TARGET_DIR)/$(RUST_LIB_NAME) $(BUILD_DIR)
	@echo "Building FFI test program..."
	$(CC) $(CFLAGS) -o $(FFI_TEST) \
		$(EXAMPLES_DIR)/test_bignum_ffi.c \
		$(TARGET_DIR)/$(RUST_LIB_NAME) \
		$(LIBS)
	@echo "✓ FFI test program built"

# Alias for backwards compatibility
test: ffi-test

# Test with memory checking (if valgrind is available)
memcheck: $(UNIT_TEST)
	@echo ""
	@echo "=== Running Memory Check ==="
	@if command -v valgrind >/dev/null 2>&1; then \
		echo "Running unit tests with valgrind..."; \
		valgrind --leak-check=full --error-exitcode=1 --track-origins=yes ./$(UNIT_TEST); \
	else \
		echo "Valgrind not available, running normal tests..."; \
		./$(UNIT_TEST); \
	fi
	@echo ""

# Clean all build artifacts
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	rm -rf $(BUILD_DIR)
	rm -f $(TEST_DIR)/*.gcov $(TEST_DIR)/*.gcno $(TEST_DIR)/*.gcda
	@echo "✓ Clean complete"

# Install the library system-wide
install: rust-lib
	@echo "Installing library and headers..."
	sudo mkdir -p /usr/local/lib /usr/local/include
	sudo cp $(TARGET_DIR)/$(RUST_LIB_NAME) /usr/local/lib/
	sudo cp $(INCLUDE_DIR)/bignum_ffi.h /usr/local/include/
	sudo ldconfig 2>/dev/null || true
	@echo "✓ Installation complete"
	@echo ""
	@echo "To use in your projects:"
	@echo "  #include <bignum_ffi.h>"
	@echo "  gcc ... -lbignum_rs -lpthread -ldl -lm"

# Show detailed project information
info:
	@echo "=== BigNum-RS Project Information ==="
	@echo ""
	@echo "Project: $(PROJECT_NAME)"
	@echo "Rust Library: $(TARGET_DIR)/$(RUST_LIB_NAME)"
	@echo "C Header: $(INCLUDE_DIR)/bignum_ffi.h"
	@echo "Unit Tests: $(TEST_DIR)/unit_test.c"
	@echo "FFI Tests: $(EXAMPLES_DIR)/test_bignum_ffi.c"
	@echo ""
	@echo "Build outputs:"
	@echo "  Unit Test: $(UNIT_TEST)"
	@echo "  FFI Test: $(FFI_TEST)"
	@echo "  Coverage Test: $(COVERAGE_TEST)"
	@echo ""
	@echo "Test Statistics:"
	@if [ -f $(UNIT_TEST) ]; then \
		echo "  Unit tests: Available"; \
	else \
		echo "  Unit tests: Not built (run 'make unit-test')"; \
	fi
	@if [ -f $(FFI_TEST) ]; then \
		echo "  FFI tests: Available"; \
	else \
		echo "  FFI tests: Not built (run 'make ffi-test')"; \
	fi
	@echo ""

# Quick test suite (unit tests only)
quick: unit-test
	@echo "✓ Quick test suite completed"

# Full test suite (all tests + coverage)
full: unit-test ffi-test coverage
	@echo "✓ Full test suite completed"

# Development target - build and test everything
dev: debug-lib unit-test
	@echo "✓ Development build completed"

# CI target - comprehensive testing for continuous integration
ci: clippy rust-lib unit-test ffi-test coverage
	@echo "✓ CI test suite completed"
	@echo ""
	@echo "Summary:"
	@echo "  ✓ Clippy linting passed"
	@echo "  ✓ Rust library built"
	@echo "  ✓ Unit tests passed"
	@echo "  ✓ FFI tests passed"
	@echo "  ✓ Coverage analysis completed"
