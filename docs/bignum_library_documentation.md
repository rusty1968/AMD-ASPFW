# AMD ASPFW Big Number Library Documentation

## Overview

The Big Number Library (`bignum.c`) is a cryptographic arithmetic library designed for the AMD Secure Processor Firmware (ASPFW). This library provides high-precision integer arithmetic operations essential for cryptographic computations in the SEV (Secure Encrypted Virtualization) environment.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Data Structures](#data-structures)
3. [Core Constants and Macros](#core-constants-and-macros)
4. [Function Categories](#function-categories)
5. [API Reference](#api-reference)
6. [Security Considerations](#security-considerations)
7. [Performance Notes](#performance-notes)
8. [Usage Examples](#usage-examples)
9. [Error Handling](#error-handling)

## Architecture Overview

### Design Philosophy

The library implements arbitrary precision arithmetic optimized for:
- **Security**: Constant-time operations where needed to prevent timing attacks
- **Memory Efficiency**: Fixed-size buffers with overflow protection
- **Performance**: Optimized algorithms for cryptographic operations
- **Embedded Systems**: Minimal memory footprint suitable for firmware environments

### Key Features

- **28-bit Radix**: Uses 28-bit digits for efficient 32-bit arithmetic
- **Fixed Size**: Maximum 80 elements (supporting up to 2240-bit numbers)
- **Barrett Reduction**: Optimized modular arithmetic
- **Dual Comparison**: Both fast and constant-time comparison functions
- **Scratch Memory**: Efficient temporary storage management

## Data Structures

### KC_BIGNUM Structure

```c
typedef struct _KC_BIGNUM {
    uint32_t   Used;                   // Number of elements in use
    uint32_t   Sign;                   // 0 = positive, 1 = negative
    uint32_t   Obfuscated;             // Security flag (unused in current implementation)
    uint32_t   Data[MAX_BN_ELEMENTS];  // 28-bit elements (80 elements max)
} KC_BIGNUM;
```

**Field Descriptions:**
- `Used`: Active length of the number (0 to MAX_BN_ELEMENTS)
- `Sign`: Sign bit (0 = positive, 1 = negative)
- `Obfuscated`: Reserved for future security features
- `Data`: Array of 28-bit values in little-endian order

### KC_BN_SCRATCH Structure

```c
typedef struct _KC_BN_SCRATCH {
    KC_BIGNUM       Barrett_u;          // Barrett reduction precomputed value
    KC_BIGNUM       BarrettTemp[3];     // Temporary storage for Barrett operations
    KC_BIGNUM       Temp[6];            // General temporary storage for division
} KC_BN_SCRATCH;
```

This structure provides workspace for complex operations, avoiding dynamic memory allocation.

## Core Constants and Macros

### Configuration Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `BN_BITS` | 28 | Bits per digit (leaves 4 bits for carry) |
| `MAX_BN_ELEMENTS` | 80 | Maximum array size (2240-bit numbers) |
| `BN_MASK` | `0x0FFFFFFF` | Mask for 28-bit values |

### Utility Macros

```c
#define BNTRIM(pBn)        // Remove leading zeros
#define BN_ZERO(pBn)       // Set to zero
#define BN_EVEN(pBn)       // Check if even
#define BN_SET_INT(Bn, Integer)  // Set to small integer value
```

### Comparison Results

```c
enum {
    BN_BIGGER  = 1,    // First operand is larger
    BN_SMALLER = 2,    // First operand is smaller  
    BN_EQUAL   = 3     // Operands are equal
};
```

## Function Categories

### 1. Data Conversion Functions

#### Input/Output Operations
- `BNLoad()` - Convert byte array to big number
- `BNStore()` - Convert big number to byte array
- `BNCopy()` - Copy one big number to another

### 2. Shift Operations

#### Digit-Level Shifts
- `BNShiftDigL()` - Left shift by digits (multiply by base^n)
- `BNShiftDigR()` - Right shift by digits (divide by base^n)

#### Bit-Level Shifts  
- `BNShiftBitsL()` - Left shift by bits
- `BNShiftBitsR()` - Right shift by bits

### 3. Arithmetic Operations

#### Basic Arithmetic
- `BNAdd()` - Addition with sign handling
- `BNSubtract()` - Subtraction with sign handling
- `BNMultiply()` - Full multiplication
- `BNMultiplyHighDigs()` - Partial multiplication (high digits only)

#### Specialized Operations
- `BNSet2Pwr()` - Set to power of 2
- `BNMod_2Pwr()` - Modulo power of 2
- `BNBitCount()` - Count significant bits

### 4. Division and Modular Arithmetic

#### Division
- `BNDiv()` - Full division with quotient and remainder

#### Barrett Reduction
- `BNBarrettSetup()` - Precompute Barrett constants
- `BNBarrettReduce()` - Fast modular reduction

### 5. Comparison Functions

#### Standard Comparison
- `BNCompare()` - Fast comparison (variable time)

#### Secure Comparison  
- `BNSecureCompare()` - Constant-time comparison

## API Reference

### Data Conversion

#### BNLoad
```c
uint32_t BNLoad(KC_BIGNUM *pBn, const uint8_t *pData, uint32_t DataLen);
```
**Purpose**: Load little-endian byte array into big number structure.

**Parameters**:
- `pBn`: Destination big number
- `pData`: Source byte array (must be 4-byte aligned)
- `DataLen`: Length in bytes (must be multiple of 4)

**Returns**: `KC_OK` on success, error code on failure

**Algorithm**: 
1. Validates input alignment and size
2. Packs 32-bit words into 28-bit digits
3. Handles expansion from 7 words to 8 digits efficiently

#### BNStore
```c
uint32_t BNStore(KC_BIGNUM *pBn, uint8_t *pData, uint32_t DataLen);
```
**Purpose**: Convert big number to little-endian byte array.

**Parameters**:
- `pBn`: Source big number
- `pData`: Destination buffer
- `DataLen`: Buffer size in bytes

**Returns**: `KC_OK` on success, error code on failure

### Arithmetic Operations

#### BNAdd
```c
uint32_t BNAdd(KC_BIGNUM *z, KC_BIGNUM *x, KC_BIGNUM *y);
```
**Purpose**: Compute z = x + y with full sign handling.

**Algorithm**:
1. Check signs to determine operation (add absolute values vs subtract)
2. Perform carry propagation across all digits
3. Handle final carry and sign determination

#### BNMultiply
```c
uint32_t BNMultiply(KC_BIGNUM *z, KC_BIGNUM *x, KC_BIGNUM *y, uint32_t Digits);
```
**Purpose**: Compute z = x × y using optimized multiplication.

**Parameters**:
- `z`: Result (must be different from x and y)
- `x`, `y`: Operands
- `Digits`: Limit result digits (0 = full precision)

**Algorithm**: Column-based multiplication with carry propagation

### Division Operations

#### BNDiv
```c
uint32_t BNDiv(KC_BN_SCRATCH *pScratch, KC_BIGNUM *z, KC_BIGNUM *a, 
               KC_BIGNUM *b, KC_BIGNUM *r);
```
**Purpose**: Compute z = a/b with remainder r using Algorithm 14.20 from HAC.

**Algorithm**:
1. Handle special cases (division by zero, small operands)
2. Normalize divisor to have high bit set
3. Estimate quotient digits using trial division
4. Correct estimates and perform subtraction

#### Barrett Reduction

Barrett reduction provides fast modular arithmetic for repeated operations with the same modulus.

##### BNBarrettSetup
```c
uint32_t BNBarrettSetup(KC_BN_SCRATCH *pScratch, KC_BIGNUM *m);
```
**Purpose**: Precompute u = b^(2k)/m for Barrett reduction.

##### BNBarrettReduce  
```c
uint32_t BNBarrettReduce(KC_BN_SCRATCH *pScratch, KC_BIGNUM *r, 
                         KC_BIGNUM *x, KC_BIGNUM *m);
```
**Purpose**: Compute r = x mod m using precomputed Barrett constants.

**Algorithm** (Algorithm 14.42 from HAC):
1. Estimate quotient using precomputed constants
2. Perform approximate reduction
3. Correct result with at most 2 subtractions

### Comparison Functions

#### BNCompare (Fast)
```c
uint32_t BNCompare(KC_BIGNUM *x, KC_BIGNUM *y);
```
**Purpose**: Compare absolute values of x and y (variable time).

**Returns**: `BN_BIGGER`, `BN_SMALLER`, or `BN_EQUAL`

#### BNSecureCompare (Constant Time)
```c
uint32_t BNSecureCompare(KC_BIGNUM *x, KC_BIGNUM *y);
```
**Purpose**: Compare absolute values in constant time to prevent timing attacks.

**Algorithm**: Uses bit manipulation to avoid conditional branches.

## Security Considerations

### Timing Attack Resistance

1. **BNSecureCompare**: Implements constant-time comparison using bit manipulation
2. **Barrett Reduction**: Provides consistent timing for modular operations
3. **Memory Access Patterns**: Designed to minimize data-dependent memory access

### Side-Channel Resistance

- **Fixed Memory Layout**: Scratch buffers prevent dynamic allocation patterns
- **Consistent Operations**: Algorithms avoid data-dependent branching where possible
- **Error Handling**: Uniform error paths prevent information leakage

### Input Validation

All functions include comprehensive input validation:
- Null pointer checks
- Buffer overflow protection  
- Alignment requirements
- Range validation

## Performance Notes

### Optimization Strategies

1. **28-bit Radix**: Optimal for 32-bit processors with carry handling
2. **Barrett Reduction**: O(n) modular reduction vs O(n²) for division
3. **Scratch Memory**: Eliminates allocation overhead
4. **Early Termination**: Special cases for zero, equal operands

### Memory Usage

- **Fixed Allocation**: Each KC_BIGNUM uses 332 bytes (80 × 4 + 12)
- **Scratch Space**: Additional 3,980 bytes for complex operations
- **No Dynamic Memory**: Suitable for embedded/firmware environments

### Algorithm Complexity

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Addition/Subtraction | O(n) | n = max(digits) |
| Multiplication | O(n²) | Classical algorithm |
| Division | O(n²) | HAC Algorithm 14.20 |
| Barrett Reduction | O(n) | After precomputation |
| Comparison | O(n) | Both fast and secure versions |

## Usage Examples

### Basic Arithmetic

```c
KC_BIGNUM a, b, result;
KC_BN_SCRATCH scratch;
uint32_t status;

// Initialize numbers
BN_SET_INT(a, 12345);
BN_SET_INT(b, 67890);

// Perform addition
status = BNAdd(&result, &a, &b);
if (status != KC_OK) {
    // Handle error
}
```

### Modular Arithmetic

```c
KC_BIGNUM x, modulus, result;
KC_BN_SCRATCH scratch;
uint32_t status;

// Setup Barrett reduction
status = BNBarrettSetup(&scratch, &modulus);
if (status != KC_OK) return status;

// Perform modular reduction
status = BNBarrettReduce(&scratch, &result, &x, &modulus);
```

### Data Conversion

```c
KC_BIGNUM number;
uint8_t data[128];
uint32_t status;

// Load from byte array
status = BNLoad(&number, data, sizeof(data));
if (status != KC_OK) return status;

// Store back to byte array  
status = BNStore(&number, data, sizeof(data));
```

## Error Handling

### Error Codes

| Code | Value | Description |
|------|-------|-------------|
| `KC_OK` | 0x00000000 | Success |
| `KCERR_BUFFER_OVERFLOW` | 0x00008002 | Insufficient output space |
| `KCERR_INVALID_PARAMETER` | 0x00008003 | Invalid input parameters |
| `KCERR_NULL_PTR` | 0x00008005 | Null pointer passed |
| `KCERR_BN_TOO_SMALL` | 0x0000800D | Result buffer too small |
| `KCERR_BAD_BIGNUMBER` | 0x0000800E | Malformed big number |
| `KCERR_DATA_LENGTH` | 0x00008019 | Invalid data length |

### Error Handling Best Practices

1. **Always Check Return Values**: All functions return status codes
2. **Validate Inputs**: Check for null pointers and valid ranges
3. **Buffer Management**: Ensure adequate space for results
4. **Cleanup**: Zero sensitive data when no longer needed

### Common Error Scenarios

- **Division by Zero**: Returns `KCERR_INVALID_PARAMETER`
- **Buffer Overflow**: Returns `KCERR_BUFFER_OVERFLOW` or `KCERR_BN_TOO_SMALL`
- **Invalid Alignment**: Returns `KCERR_DATA_LENGTH`
- **Null Pointers**: Returns `KCERR_NULL_PTR`

## Integration Notes

### Dependencies

- **Standard C Library**: `stdio.h`, `string.h`
- **Common Utilities**: `common_utilities.h` (for arraysize macro)
- **No Dynamic Allocation**: All memory statically allocated

### Thread Safety

- **Not Thread-Safe**: Functions are not reentrant
- **Scratch Buffers**: Must not be shared between threads
- **Atomic Operations**: Not required due to firmware environment

### Compiler Considerations

- **Optimization**: Designed for -O2 optimization level
- **Alignment**: Assumes natural alignment for uint32_t
- **Endianness**: Little-endian byte order assumed

## Conclusion

The AMD ASPFW Big Number Library provides a robust, secure foundation for cryptographic arithmetic operations in embedded firmware environments. Its design prioritizes security, performance, and reliability while maintaining a minimal memory footprint suitable for secure processor applications.

The library's implementation of standard algorithms (HAC), security features (constant-time operations), and comprehensive error handling make it suitable for production cryptographic applications requiring high assurance.

---

**Document Version**: 1.0  
**Last Updated**: August 1, 2025  
**Author**: AMD ASPFW Team  
**License**: AMD Proprietary / ISC License (libsodium portions)
