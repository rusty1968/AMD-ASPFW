# Reverse Engineering Security Requirements from Original C Code

**Document**: Security Requirements Analysis for BigNum-RS  
**Date**: August 5, 2025  
**Context**: LLM Code Generation Assessment for Cryptographic Firmware  

---

## Executive Summary

This document analyzes how security requirements for the BigNum-RS project were **reverse-engineered from the original AMD ASPFW C code** rather than assumed or fabricated. The analysis demonstrates that constant-time operation requirements and cryptographic security patterns were explicitly documented and implemented in the original codebase.

**Key Finding**: The original AMD code provides clear evidence of security-aware design through dual implementations, explicit timing annotations, and usage patterns in cryptographic contexts.

---

## 1. Evidence-Based Security Requirements Discovery

### 1.1 Explicit Code Documentation

The original `bignum.c` file contains explicit documentation of timing characteristics:

**Lines 662-672: Non-Constant Time Implementation**
```c
/**
 *  Compares the absolute value of x against y.
 *  Fast/Non-constant time                    ← EXPLICIT timing annotation
 *
 *  Parameters:
 *      x - first number to compare
 *      y - number to comare against
 *
 *  Return value:
 *   x > y  - BN_BIGGER
 *   x < y  - BN_SMALLER
 *   x == y - BN_EQUAL
 */
uint32_t BNCompare(KC_BIGNUM *x, KC_BIGNUM *y)
```

**Lines 709-725: Constant Time Implementation**
```c
/**
 *  Compares the absolute value of x against y.
 *  Slow/Constant time                        ← EXPLICIT timing annotation
 *  >> 31 because each element is uint32_t (technically each element is BN_BITS)
 *
 *  Borrowed under ISC license from
 *      https://github.com/jedisct1/libsodium    ← Attribution to crypto library
 *      https://github.com/jedisct1/libsodium/blob/master/LICENSE
 *
 *  Parameters:
 *      x - first number to compare
 *      y - number to comare against
 *
 *  Return value:
 *   x > y  - BN_BIGGER
 *   x < y  - BN_SMALLER
 *   x == y - BN_EQUAL
 */
uint32_t BNSecureCompare(KC_BIGNUM *x, KC_BIGNUM *y)
```

### 1.2 Implementation Evidence

**Non-Constant Time (Variable Timing) Implementation**
```c
uint32_t BNCompare(KC_BIGNUM *x, KC_BIGNUM *y)
{
    // Early exit on size difference - timing leak
    if (x->Used > y->Used)
        return BN_BIGGER;
    if (x->Used < y->Used)
        return BN_SMALLER;
    
    // Variable loop length based on Used field
    for (i = x->Used - 1; i >= 0; i--) {
        if (pX[i] > pY[i])
            return BN_BIGGER;    // Early exit reveals comparison position
        if (pX[i] < pY[i])
            return BN_SMALLER;
    }
    return BN_EQUAL;
}
```

**Constant Time Implementation**
```c
uint32_t BNSecureCompare(KC_BIGNUM *x, KC_BIGNUM *y)
{
    size_t i = MAX_BN_ELEMENTS;    // Always process all elements
    uint32_t *pX = &x->Data[0];
    uint32_t *pY = &y->Data[0];
    uint32_t gt = 0;
    uint32_t eq = 1;

    // Always compare ALL elements regardless of Used field
    while (i != 0) {
        i--;
        gt |= ((pY[i] - pX[i]) >> 31) & eq;    // Bit manipulation without branches
        eq &= ((pY[i] ^ pX[i]) - 1) >> 31;
    }
    return (2 + (eq - gt));
}
```

---

## 2. Cryptographic Context Evidence

### 2.1 Usage Pattern Analysis

The secure comparison function is used specifically in **cryptographic contexts**:

**Line 1326: Barrett Modular Reduction**
```c
// Don't have to do anything if x is smaller than modulus.
if (BN_SMALLER == BNSecureCompare(x, m))
{
    BNCopy(r, x);
    return KC_OK;
}
```

**Lines 1376, 1399: Modular Arithmetic Operations**
```c
Compare = BNSecureCompare(r, m);

k = 0;        // use it as counter now

// while r >= m, do: r = r - m
```

### 2.2 Cryptographic Algorithm Context

The functions are used in **Barrett modular reduction** (Algorithm 14.42 from Handbook of Applied Cryptography):

```c
/**
 *  Barrett modular reduction, algorithm 14.42 from HAC:
 *  Calculate u = b**2k / m for Barrett modular reduction.
 */
uint32_t BNBarrettSetup(KC_BN_SCRATCH *pScratch, KC_BIGNUM *m)

/**
 *  Barrett modular reduction, algorithm 14.42 from HAC:
 */
uint32_t BNBarrettReduce(KC_BN_SCRATCH *pScratch, KC_BIGNUM *r, KC_BIGNUM *x, KC_BIGNUM *m)
```

---

## 3. Security Anti-Patterns Discovered

### 3.1 Information Leakage Through Error Codes

**Line 154: Detailed Error Context Leak**
```c
if (DataLen == 0)
{
    status = KCERR_BUFFER_OVERFLOW + 0xB00;    // Magic number reveals internal state
    break;
}
```

**Lines 69-74: Size-Revealing Error Patterns**
```c
/* Input data length must be multiple of 4 */
if (DataLen & 0x03)
    return KCERR_DATA_LENGTH;        // Reveals alignment requirement

/* Check if data is not bigger than max BigNum size */
if ((BnLen + 2) > arraysize(pBn->Data))
    return KCERR_DATA_LENGTH;        // Reveals size calculation details
```

### 3.2 Timing Attack Vulnerabilities

**Variable-Time Normalization (Lines 32-36)**
```c
#define BNTRIM(pBn)                                             \
{                                                               \
    while ((pBn->Used != 0) && (pBn->Data[pBn->Used-1] == 0))   \
        pBn->Used--;    // Loop count reveals number of leading zeros
}
```

### 3.3 Missing Input Validation

**Lines 186-189: Missing Null Pointer Checks**
```c
void BNCopy(KC_BIGNUM *pDest, KC_BIGNUM *pSrc)
{
    if (pDest != pSrc)    // Only checks inequality, not null
        memcpy(pDest, pSrc, sizeof(*pDest));    // Potential null dereference
}
```

---

## 4. AMD ASPFW Security Context

### 4.1 Security Processor Firmware

This code is part of **AMD's ASPFW (AMD Secure Processor Firmware)** - a security coprocessor designed for:
- Cryptographic operations
- Secure boot processes
- Hardware security module functions
- Side-channel attack resistance

### 4.2 Embedded Security Requirements

The firmware operates in a **security-critical embedded environment** where:
- Timing attacks are feasible and dangerous
- Information leakage can compromise cryptographic keys
- Constant-time operations are essential for security
- Resource constraints limit defensive options

---

## 5. Libsodium Attribution Evidence

### 5.1 External Cryptographic Library Reference

The constant-time implementation includes explicit attribution:

```c
 *  Borrowed under ISC license from
 *      https://github.com/jedisct1/libsodium
 *      https://github.com/jedisct1/libsodium/blob/master/LICENSE
```

**Significance**: Libsodium is a well-known cryptographic library specifically designed for:
- Constant-time operations
- Side-channel attack resistance
- Secure cryptographic implementations

This attribution provides **external validation** that the AMD developers were specifically addressing timing attack concerns.

---

## 6. Pattern Inheritance Risk Analysis

### 6.1 The LLM Translation Challenge

**Problem**: Direct LLM translation would inherit the **non-secure patterns** from the original C code:

1. **Default Function Selection**: LLM would naturally translate `BNCompare` (fast/non-constant) rather than `BNSecureCompare` (slow/constant)
2. **Error Pattern Preservation**: Information-leaking error codes would be translated directly
3. **Timing Vulnerability Inheritance**: Variable-time algorithms would be preserved in Rust

### 6.2 Security-Aware Design Requirement

**Solution**: The BigNum-RS project required **explicit security-aware design**:

```rust
// ❌ DIRECT TRANSLATION: Inherits timing vulnerabilities
pub extern "C" fn BNCompare(x: *const KC_BIGNUM, y: *const KC_BIGNUM) -> u32 {
    let x = unsafe { &*x };
    let y = unsafe { &*y };
    
    // Same early exit pattern as C - timing side-channel
    if x.Used != y.Used {
        return if x.Used > y.Used { BN_BIGGER } else { BN_SMALLER };
    }
    // ... variable-time comparison
}

// ✅ SECURITY-AWARE IMPLEMENTATION: Constant-time comparison
pub extern "C" fn BNSecureCompare(x: *const KC_BIGNUM, y: *const KC_BIGNUM) -> u32 {
    let x = unsafe { &*x };
    let y = unsafe { &*y };
    
    // Always compare ALL elements regardless of Used field - timing safe
    let equal = constant_time_compare(x, y);
    if equal.into() { BN_EQUAL } else { BN_NOT_EQUAL }
}
```

---

## 7. Validation of Requirements Engineering

### 7.1 Requirements Source Verification

| Requirement | Source | Evidence Location | Type |
|-------------|--------|-------------------|------|
| Constant-time operations | Code comments | Lines 662, 709 | Explicit documentation |
| Timing attack resistance | Implementation | `BNSecureCompare` function | Implementation proof |
| Cryptographic usage | Function calls | Lines 1326, 1376, 1399 | Usage context |
| External validation | Attribution | Libsodium reference | Third-party validation |
| Security processor context | File location | AMD ASPFW repository | System architecture |

### 7.2 Anti-Pattern Documentation

| Anti-Pattern | Location | Security Impact | Evidence Type |
|--------------|----------|-----------------|---------------|
| Information leakage | Line 154 | Reveals internal state | Error code analysis |
| Timing vulnerabilities | `BNCompare` | Position/size leakage | Algorithm analysis |
| Missing validation | Lines 186-189 | Memory safety | Code inspection |
| Variable-time normalization | Lines 32-36 | Leading zero count leak | Macro analysis |

---

## 8. Methodology: Reverse Engineering vs. Assumption

### 8.1 Evidence-Based Analysis Process

1. **Code Documentation Review**: Identified explicit timing annotations
2. **Implementation Comparison**: Analyzed dual implementations (fast vs. secure)
3. **Usage Pattern Analysis**: Traced function calls in cryptographic contexts
4. **External Reference Validation**: Verified libsodium attribution
5. **Security Context Research**: AMD ASPFW security processor purpose
6. **Anti-Pattern Identification**: Documented problematic patterns in original code

### 8.2 No Assumptions Made

**What was NOT assumed**:
- Generic "crypto should be constant-time" principles
- Industry best practices without evidence
- Theoretical security requirements
- Modern cryptographic standards applied retroactively

**What was DISCOVERED**:
- Explicit developer documentation of timing characteristics
- Implemented dual-path design for timing-sensitive operations
- Actual usage in cryptographic algorithm implementations
- Reference to established cryptographic library (libsodium)

---

## 9. Implications for LLM-Based Migration

### 9.1 Pattern Inheritance Risk

The analysis demonstrates that **LLM translation inherits existing security vulnerabilities** rather than automatically improving security. Key risks:

1. **Default to Non-Secure Implementation**: LLM would naturally choose `BNCompare` over `BNSecureCompare`
2. **Error Pattern Preservation**: Information-leaking patterns would be translated directly
3. **Security Context Loss**: The reason for dual implementations might not be preserved

### 9.2 Expert Validation Requirement

This analysis validates the need for **cryptographic expert oversight** in LLM-based migration:

- **Security requirements must be explicitly identified** from original code
- **Secure variants must be deliberately selected** over convenient implementations
- **Anti-patterns must be recognized and corrected** rather than translated

---

## 10. Conclusion

### 10.1 Requirements Engineering Validation

The constant-time operation requirements for BigNum-RS were **reverse-engineered from explicit evidence** in the original AMD ASPFW code:

✅ **Documented**: Explicit timing annotations in function comments  
✅ **Implemented**: Dual implementations with different timing characteristics  
✅ **Used**: Secure variants employed in cryptographic contexts  
✅ **Validated**: External reference to established cryptographic library  
✅ **Contextualized**: Security processor firmware environment  

### 10.2 LLM Migration Insight

This analysis demonstrates a **critical challenge in LLM-based cryptographic code migration**:

**Memory Safety ≠ Cryptographic Security**

LLM translation provides memory safety improvements but **inherits timing vulnerabilities and information leakage patterns** from the original code unless explicitly guided by cryptographic domain expertise.

### 10.3 Methodology Validation

The BigNum-RS project demonstrates that **evidence-based security requirements engineering** is essential for successful LLM-assisted migration of cryptographic code. Security requirements cannot be assumed - they must be discovered, documented, and explicitly addressed in the migration process.

---

## Appendix: Evidence Locations

### Source File References
- **File**: `/home/ferrite/rusty1968/AMD-ASPFW/fw/psp_bl_uapps/sev_uapp/src/bignum.c`
- **Total Lines**: 1,407
- **Key Evidence Locations**:
  - Lines 662-672: Non-constant time documentation
  - Lines 709-725: Constant time documentation and implementation
  - Lines 1326, 1376, 1399: Cryptographic usage contexts
  - Line 154: Information leakage anti-pattern
  - Lines 32-36: Timing-dependent normalization

### Cross-Reference Files
- **bignum.h**: Structure definitions and function declarations
- **Context**: AMD ASPFW security processor firmware repository
- **Domain**: Cryptographic embedded systems firmware

**Analysis Date**: August 5, 2025  
**Methodology**: Evidence-based reverse engineering of security requirements from original source code
