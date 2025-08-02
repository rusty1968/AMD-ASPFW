# Makefile for bignum-rs FFI library and examples
# This builds the Rust library and C test programs

# Project configuration
PROJECT_NAME = bignum_rs
RUST_LIB_NAME = lib$(PROJECT_NAME).a
SHARED_LIB_NAME = lib$(PROJECT_NAME).so

# Directories
SRC_DIR = src
EXAMPLES_DIR = examples
INCLUDE_DIR = include
TARGET_DIR = target/release
BUILD_DIR = build

# Compiler and flags
CC = gcc
CFLAGS = -Wall -Wextra -O2 -I$(INCLUDE_DIR)
LDFLAGS = -L$(TARGET_DIR) -l$(PROJECT_NAME) -lpthread -ldl

# Rust configuration
CARGO = cargo
CARGO_FLAGS = --release

# Default target
.PHONY: all
all: rust-lib test-program

# Build the Rust library
.PHONY: rust-lib
rust-lib:
	@echo "Building Rust library..."
	$(CARGO) build $(CARGO_FLAGS)
	@echo "Rust library built successfully"

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Build the test program
.PHONY: test-program
test-program: rust-lib $(BUILD_DIR)
	@echo "Building C test program..."
	$(CC) $(CFLAGS) -o $(BUILD_DIR)/test_bignum_ffi \
		$(EXAMPLES_DIR)/test_bignum_ffi.c \
		$(TARGET_DIR)/$(RUST_LIB_NAME) \
		-lpthread -ldl -lm
	@echo "Test program built successfully"

# Run the test program
.PHONY: test
test: test-program
	@echo "Running FFI tests..."
	@echo "========================"
	./$(BUILD_DIR)/test_bignum_ffi
	@echo "========================"

# Generate documentation
.PHONY: docs
docs:
	@echo "Generating Rust documentation..."
	$(CARGO) doc --no-deps --open

# Check Rust code
.PHONY: check
check:
	@echo "Checking Rust code..."
	$(CARGO) check
	$(CARGO) clippy -- -D warnings
	$(CARGO) fmt --check

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	rm -rf $(BUILD_DIR)
	@echo "Clean complete"

# Install the library (system-wide)
.PHONY: install
install: rust-lib
	@echo "Installing library and headers..."
	sudo cp $(TARGET_DIR)/$(RUST_LIB_NAME) /usr/local/lib/
	sudo cp $(TARGET_DIR)/$(SHARED_LIB_NAME) /usr/local/lib/ 2>/dev/null || true
	sudo cp $(INCLUDE_DIR)/bignum_rs_ffi.h /usr/local/include/
	sudo ldconfig
	@echo "Installation complete"

# Uninstall the library
.PHONY: uninstall
uninstall:
	@echo "Uninstalling library and headers..."
	sudo rm -f /usr/local/lib/$(RUST_LIB_NAME)
	sudo rm -f /usr/local/lib/$(SHARED_LIB_NAME)
	sudo rm -f /usr/local/include/bignum_rs_ffi.h
	sudo ldconfig
	@echo "Uninstall complete"

# Show library information
.PHONY: info
info:
	@echo "=== BigNum-RS Build Information ==="
	@echo "Project: $(PROJECT_NAME)"
	@echo "Static lib: $(TARGET_DIR)/$(RUST_LIB_NAME)"
	@echo "Shared lib: $(TARGET_DIR)/$(SHARED_LIB_NAME)"
	@echo "Headers: $(INCLUDE_DIR)/bignum_rs_ffi.h"
	@echo "Test program: $(BUILD_DIR)/test_bignum_ffi"
	@echo ""
	@echo "Available targets:"
	@echo "  all         - Build library and test program"
	@echo "  rust-lib    - Build only the Rust library"
	@echo "  test-program- Build only the C test program"
	@echo "  test        - Build and run tests"
	@echo "  docs        - Generate Rust documentation"
	@echo "  check       - Run Rust linting and formatting checks"
	@echo "  clean       - Clean all build artifacts"
	@echo "  install     - Install library system-wide"
	@echo "  uninstall   - Remove installed library"
	@echo "  info        - Show this information"

# Integration example for SEV firmware
.PHONY: integration-example
integration-example:
	@echo "=== Integration with SEV Firmware ==="
	@echo ""
	@echo "To integrate bignum-rs with the SEV firmware:"
	@echo ""
	@echo "1. Copy the static library:"
	@echo "   cp $(TARGET_DIR)/$(RUST_LIB_NAME) /path/to/sev/lib/"
	@echo ""
	@echo "2. Copy the header file:"
	@echo "   cp $(INCLUDE_DIR)/bignum_rs_ffi.h /path/to/sev/include/"
	@echo ""
	@echo "3. Update the SEV build system to link with bignum-rs:"
	@echo "   # In your Makefile or build script:"
	@echo "   LIBS += -lbignum_rs -lpthread -ldl"
	@echo ""
	@echo "4. Replace bignum.c includes:"
	@echo "   # Change:"
	@echo "   #include \"bignum.h\""
	@echo "   # To:"
	@echo "   #include \"bignum_rs_ffi.h\""
	@echo ""
	@echo "5. The API remains identical - no code changes needed!"
	@echo ""

# Benchmark target (future enhancement)
.PHONY: bench
bench:
	@echo "Running Rust benchmarks..."
	$(CARGO) bench

# Profile target for performance analysis
.PHONY: profile
profile: CARGO_FLAGS += --profile profiling
profile: rust-lib test-program
	@echo "Built with profiling enabled"
	@echo "Run: perf record -g ./$(BUILD_DIR)/test_bignum_ffi"
	@echo "Then: perf report"

.PHONY: help
help: info
