# AMD ASPFW Big Number Library: Architecture Block Diagrams

## Overview

This document provides visual representations of the AMD ASPFW Big Number Library architecture, including the current C implementation and proposed Rust migration designs. The diagrams illustrate module relationships, data flow, and architectural decisions for both implementations.

## Table of Contents

1. [Current C Implementation Architecture](#current-c-implementation-architecture)
2. [Rust Migration Architecture Options](#rust-migration-architecture-options)
3. [Data Structure Relationships](#data-structure-relationships)
4. [Operation Flow Diagrams](#operation-flow-diagrams)
5. [Security Module Architecture](#security-module-architecture)
6. [Memory Layout Diagrams](#memory-layout-diagrams)

## Current C Implementation Architecture

### High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    AMD ASPFW SEV Firmware                          │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐     │
│  │   ECC Module    │  │   RSA Module    │  │  AES Module     │     │
│  │                 │  │                 │  │                 │     │
│  └─────────┬───────┘  └─────────┬───────┘  └─────────────────┘     │
│            │                    │                                  │
│            └────────────────────┼──────────────────────────────────┤
│                                 │                                  │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Big Number Library (bignum.c)                  │   │
│  │                                                             │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │   │
│  │  │Data Convert │  │ Arithmetic  │  │  Security Functions │ │   │
│  │  │   Module    │  │   Module    │  │                     │ │   │
│  │  │             │  │             │  │ • BNSecureCompare   │ │   │
│  │  │ • BNLoad    │  │ • BNAdd     │  │ • Constant-time ops │ │   │
│  │  │ • BNStore   │  │ • BNSub     │  │ • Side-channel      │ │   │
│  │  │ • BNCopy    │  │ • BNMul     │  │   resistance        │ │   │
│  │  │             │  │ • BNDiv     │  │                     │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ │   │
│  │                                                             │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │   │
│  │  │   Shift     │  │   Modular   │  │    Comparison       │ │   │
│  │  │  Operations │  │ Arithmetic  │  │    Functions        │ │   │
│  │  │             │  │             │  │                     │ │   │
│  │  │ • BNShiftL  │  │ • Barrett   │  │ • BNCompare         │ │   │
│  │  │ • BNShiftR  │  │   Reduction │  │ • BNSecureCompare   │ │   │
│  │  │ • Bit ops   │  │ • BNMod2Pwr │  │                     │ │   │
│  │  │             │  │             │  │                     │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                Hardware Abstraction Layer                   │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### Big Number Library Internal Architecture

```
┌───────────────────────────────────────────────────────────────────────┐
│                    Big Number Library Core                           │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                    Data Structures Layer                        │  │
│  │                                                                 │  │
│  │  KC_BIGNUM Structure         KC_BN_SCRATCH Structure            │  │
│  │  ┌─────────────────┐         ┌─────────────────────────────┐    │  │
│  │  │ Used: u32       │         │ Barrett_u: KC_BIGNUM       │    │  │
│  │  │ Sign: u32       │         │ BarrettTemp[3]: KC_BIGNUM  │    │  │
│  │  │ Obfuscated: u32 │         │ Temp[6]: KC_BIGNUM         │    │  │
│  │  │ Data[80]: u32   │         │                             │    │  │
│  │  │ (28-bit each)   │         │ (Workspace for complex ops) │    │  │
│  │  └─────────────────┘         └─────────────────────────────┘    │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                     Function Categories                         │  │
│  │                                                                 │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │  │
│  │  │   Input/    │  │  Arithmetic │  │      Security           │  │
│  │  │   Output    │  │ Operations  │  │    Functions            │  │
│  │  │             │  │             │  │                         │  │
│  │  │ BNLoad()    │  │ BNAdd()     │  │ BNSecureCompare()       │  │
│  │  │ BNStore()   │  │ BNSubtract()│  │ Constant-time ops       │  │
│  │  │ BNCopy()    │  │ BNMultiply()│  │ Timing attack resist.   │  │
│  │  │             │  │ BNDiv()     │  │                         │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │  │
│  │                                                                 │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │  │
│  │  │    Shift    │  │   Modular   │  │     Comparison          │  │
│  │  │ Operations  │  │ Arithmetic  │  │    Functions            │  │
│  │  │             │  │             │  │                         │  │
│  │  │BNShiftDigL()│  │BNBarrettSet │  │ BNCompare() (fast)      │  │
│  │  │BNShiftDigR()│  │ up()        │  │ BNSecureCompare()       │  │
│  │  │BNShiftBitsL │  │BNBarrettRed │  │ (constant-time)         │  │
│  │  │BNShiftBitsR │  │ uce()       │  │                         │  │
│  │  │BNBitCount() │  │BNMod_2Pwr() │  │                         │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                 Algorithm Implementations                       │  │
│  │                                                                 │  │
│  │  • HAC Algorithm 14.20 (Division)                              │  │
│  │  • HAC Algorithm 14.42 (Barrett Reduction)                     │  │
│  │  • 28-bit Radix Arithmetic                                     │  │
│  │  • Carry Propagation Optimization                              │  │
│  │  • Constant-time Bit Manipulation                              │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────┘
```

## Rust Migration Architecture Options

### Option 1: Direct Replacement with crypto-bigint

```
┌───────────────────────────────────────────────────────────────────────┐
│                    Rust Big Number Library                           │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                  Compatibility Layer                            │  │
│  │                                                                 │  │
│  │  ┌─────────────────┐         ┌─────────────────────────────┐    │  │
│  │  │   KcBignum      │         │    KcBnScratch              │    │  │
│  │  │   (Wrapper)     │         │    (Wrapper)                │    │  │
│  │  │                 │         │                             │    │  │
│  │  │ used: u32       │         │ barrett_temp: [KcBignum;3]  │    │  │
│  │  │ sign: u32       │         │ temp: [KcBignum; 6]         │    │  │
│  │  │ obfuscated: u32 │         │                             │    │  │
│  │  │ inner: U2048    │◄────────┤ (Maps to crypto-bigint)     │    │  │
│  │  └─────────────────┘         └─────────────────────────────┘    │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                    crypto-bigint Core                           │  │
│  │                                                                 │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │  │
│  │  │ Arithmetic  │  │ Constant-   │  │    Memory Management    │  │
│  │  │ Operations  │  │ Time Ops    │  │                         │  │
│  │  │             │  │             │  │ • Fixed-size arrays     │  │
│  │  │ • Add/Sub   │  │ • ct_eq()   │  │ • No heap allocation    │  │
│  │  │ • Mul/Div   │  │ • ct_lt()   │  │ • Stack-based           │  │
│  │  │ • Modular   │  │ • ct_gt()   │  │                         │  │
│  │  │   ops       │  │             │  │                         │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                   Limitations & Gaps                            │  │
│  │                                                                 │  │
│  │  ❌ Different radix (64-bit vs 28-bit)                         │  │
│  │  ❌ Fixed size constraints (U2048, U4096)                      │  │
│  │  ❌ No Barrett reduction implementation                         │  │
│  │  ⚠️  Conversion overhead between representations               │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────┘
```

### Option 2: Custom Rust Implementation

```
┌───────────────────────────────────────────────────────────────────────┐
│                 Custom Rust Big Number Library                       │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                 Exact C Compatibility                           │  │
│  │                                                                 │  │
│  │  #[repr(C)]                   #[repr(C)]                        │  │
│  │  struct KcBignum {            struct KcBnScratch {              │  │
│  │    used: u32,                   barrett_u: KcBignum,            │  │
│  │    sign: u32,                   barrett_temp: [KcBignum; 3],    │  │
│  │    obfuscated: u32,             temp: [KcBignum; 6],            │  │
│  │    data: [u32; 80],           }                                 │  │
│  │  }                                                              │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │              28-bit Arithmetic Implementation                   │  │
│  │                                                                 │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │  │
│  │  │   Basic     │  │  Advanced   │  │     Specialized         │  │
│  │  │ Operations  │  │ Operations  │  │    Operations           │  │
│  │  │             │  │             │  │                         │  │
│  │  │ • add_impl  │  │ • multiply  │  │ • barrett_setup         │  │
│  │  │ • sub_impl  │  │ • divide    │  │ • barrett_reduce        │  │
│  │  │ • shift_ops │  │ • modular   │  │ • secure_compare        │  │
│  │  │             │  │   ops       │  │                         │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                Security Implementation                          │  │
│  │                                                                 │  │
│  │  ┌─────────────────────────────────────────────────────────┐    │  │
│  │  │              Constant-Time Operations                   │    │  │
│  │  │                                                         │    │  │
│  │  │  • No data-dependent branching                          │    │  │
│  │  │  • Bit manipulation for comparisons                     │    │  │
│  │  │  • Uniform memory access patterns                       │    │  │
│  │  │  • libsodium-style secure_compare                       │    │  │
│  │  └─────────────────────────────────────────────────────────┘    │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                     Advantages                                  │  │
│  │                                                                 │  │
│  │  ✅ Exact memory layout match                                  │  │
│  │  ✅ Preserves 28-bit optimization                              │  │
│  │  ✅ Full control over algorithms                               │  │
│  │  ✅ Rust memory safety benefits                                │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────┘
```

### Option 3: Hybrid Approach (Recommended)

```
┌───────────────────────────────────────────────────────────────────────┐
│                    Hybrid Rust Big Number Library                    │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                  Public API Layer                               │  │
│  │                                                                 │  │
│  │  C-Compatible Interface          Rust Native Interface          │  │
│  │  ┌─────────────────┐             ┌─────────────────────────┐     │  │
│  │  │ #[repr(C)]      │             │ impl KcBignum           │     │  │
│  │  │ KcBignum        │◄──────────► │                         │     │  │
│  │  │                 │             │ • load() -> Result<>    │     │  │
│  │  │ • bn_load()     │             │ • add() -> Result<>     │     │  │
│  │  │ • bn_add()      │             │ • multiply() -> Result<>│     │  │
│  │  │ • bn_compare()  │             │                         │     │  │
│  │  └─────────────────┘             └─────────────────────────┘     │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                   Operation Routing Layer                       │  │
│  │                                                                 │  │
│  │              ┌─────────────────────────────────┐                │  │
│  │              │        Operation Router        │                │  │
│  │              │                                 │                │  │
│  │              │  if simple_operation {          │                │  │
│  │              │    use_28bit_impl()             │                │  │
│  │              │  } else {                       │                │  │
│  │              │    use_crypto_bigint()          │                │  │
│  │              │  }                              │                │  │
│  │              └─────────────────────────────────┘                │  │
│  │                            │                                    │  │
│  │        ┌───────────────────┼───────────────────┐                │  │
│  │        │                   │                   │                │  │
│  │        ▼                   ▼                   ▼                │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                Implementation Backends                          │  │
│  │                                                                 │  │
│  │  ┌─────────────────┐     ┌───────────────┐     ┌──────────────┐ │  │
│  │  │   28-bit Native │     │ crypto-bigint │     │  Conversion  │ │  │
│  │  │  Implementation │     │   Backend     │     │   Module     │ │  │
│  │  │                 │     │               │     │              │ │  │
│  │  │ • Fast add/sub  │     │ • Secure mul  │     │ • 28↔64 bit  │ │  │
│  │  │ • Bit ops       │     │ • Secure div  │     │ • Format     │ │  │
│  │  │ • Shifts        │     │ • Const-time  │     │   transform  │ │  │
│  │  │ • Data convert  │     │   compare     │     │              │ │  │
│  │  │                 │     │ • Modular ops │     │              │ │  │
│  │  └─────────────────┘     └───────────────┘     └──────────────┘ │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                   Security Guarantees                           │  │
│  │                                                                 │  │
│  │  ✅ Constant-time for security-critical operations              │  │
│  │  ✅ Fast path for performance-critical operations               │  │
│  │  ✅ Memory safety from Rust ownership system                    │  │
│  │  ✅ Exact C compatibility for firmware integration              │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────┘
```

## Data Structure Relationships

### Current C Implementation

```
KC_BIGNUM Structure Layout:
┌─────────────────────────────────────┐
│ Field        │ Size    │ Offset     │
├─────────────────────────────────────┤
│ Used         │ 4 bytes │ 0          │
│ Sign         │ 4 bytes │ 4          │
│ Obfuscated   │ 4 bytes │ 8          │
│ Data[0]      │ 4 bytes │ 12         │
│ Data[1]      │ 4 bytes │ 16         │
│ ...          │ ...     │ ...        │
│ Data[79]     │ 4 bytes │ 328        │
└─────────────────────────────────────┘
Total Size: 332 bytes

KC_BN_SCRATCH Structure:
┌─────────────────────────────────────┐
│ Barrett_u    │ 332 bytes │ 0       │
│ BarrettTemp[0]│ 332 bytes │ 332     │
│ BarrettTemp[1]│ 332 bytes │ 664     │
│ BarrettTemp[2]│ 332 bytes │ 996     │
│ Temp[0]      │ 332 bytes │ 1328    │
│ Temp[1]      │ 332 bytes │ 1660    │
│ Temp[2]      │ 332 bytes │ 1992    │
│ Temp[3]      │ 332 bytes │ 2324    │
│ Temp[4]      │ 332 bytes │ 2656    │
│ Temp[5]      │ 332 bytes │ 2988    │
└─────────────────────────────────────┘
Total Size: 3,320 bytes
```

### Rust Hybrid Implementation

```
Data Structure Mapping:

┌─────────────────────────────────────────────────────────────────┐
│                        KcBignum                                 │
│                                                                 │
│  ┌─────────────────┐      ┌─────────────────────────────────┐   │
│  │  C-Compatible   │      │     Internal Representation    │   │
│  │   Layout        │      │                                 │   │
│  │                 │      │                                 │   │
│  │ used: u32       │      │ ┌─────────────────────────────┐ │   │
│  │ sign: u32       │◄────►│ │     28-bit Data Array       │ │   │
│  │ obfuscated: u32 │      │ │                             │ │   │
│  │ data: [u32; 80] │      │ │ [u32; 80] where each        │ │   │
│  │                 │      │ │ element uses 28 bits        │ │   │
│  │                 │      │ │                             │ │   │
│  └─────────────────┘      │ └─────────────────────────────┘ │   │
│                           │                                 │   │
│                           │ ┌─────────────────────────────┐ │   │
│                           │ │  crypto-bigint Buffer       │ │   │
│                           │ │                             │ │   │
│                           │ │ Temporary conversion for    │ │   │
│                           │ │ complex operations          │ │   │
│                           │ │                             │ │   │
│                           │ └─────────────────────────────┘ │   │
│                           └─────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Operation Flow Diagrams

### Basic Addition Operation Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Addition Operation Flow                       │
└─────────────────────────────────────────────────────────────────────┘

Input: KcBignum A, KcBignum B
│
▼
┌─────────────────────────────────────┐
│        Input Validation             │
│                                     │
│ • Check null pointers               │
│ • Validate Used field bounds        │
│ • Check for malformed numbers       │
└─────────────────┬───────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│       Sign Analysis                 │
│                                     │
│ if (A.sign == B.sign) {             │
│   operation = ADD_ABSOLUTE          │
│ } else {                            │
│   operation = SUBTRACT_ABSOLUTE     │
│ }                                   │
└─────────────────┬───────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│     28-bit Addition Loop            │
│                                     │
│ carry = 0                           │
│ for i in 0..max(A.used, B.used) {  │
│   sum = A.data[i] + B.data[i] + carry│
│   result.data[i] = sum & BN_MASK    │
│   carry = sum >> BN_BITS            │
│ }                                   │
└─────────────────┬───────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│      Carry Propagation              │
│                                     │
│ if (carry > 0 && space_available) { │
│   result.data[max_len] = carry      │
│   result.used = max_len + 1         │
│ }                                   │
└─────────────────┬───────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│         Normalization               │
│                                     │
│ • Remove leading zeros (BNTRIM)     │
│ • Set appropriate sign bit          │
│ • Update Used field                 │
└─────────────────┬───────────────────┘
                  │
                  ▼
Output: Result KcBignum
```

### Complex Multiplication Flow (Hybrid Approach)

```
┌─────────────────────────────────────────────────────────────────────┐
│                   Multiplication Operation Flow                     │
└─────────────────────────────────────────────────────────────────────┘

Input: KcBignum A, KcBignum B
│
▼
┌─────────────────────────────────────┐
│        Size Analysis                │
│                                     │
│ if (A.used < SMALL_THRESHOLD &&     │
│     B.used < SMALL_THRESHOLD) {     │
│   method = NATIVE_28BIT             │
│ } else {                            │
│   method = CRYPTO_BIGINT            │
│ }                                   │
└─────────────────┬───────────────────┘
                  │
        ┌─────────┴─────────┐
        │                   │
        ▼                   ▼
┌─────────────────┐ ┌─────────────────────────────────┐
│  Native 28-bit  │ │    crypto-bigint Path          │
│  Multiplication │ │                                 │
│                 │ │ ┌─────────────────────────────┐ │
│ Column-based    │ │ │    Convert to U2048         │ │
│ multiplication  │ │ │                             │ │
│ with carry      │ │ │ A_cb = convert_to_crypto(A) │ │
│ propagation     │ │ │ B_cb = convert_to_crypto(B) │ │
│                 │ │ └─────────────┬───────────────┘ │
│                 │ │               │                 │
│                 │ │               ▼                 │
│                 │ │ ┌─────────────────────────────┐ │
│                 │ │ │  Secure Multiplication      │ │
│                 │ │ │                             │ │
│                 │ │ │ result_cb = A_cb * B_cb     │ │
│                 │ │ │ (constant-time operation)   │ │
│                 │ │ └─────────────┬───────────────┘ │
│                 │ │               │                 │
│                 │ │               ▼                 │
│                 │ │ ┌─────────────────────────────┐ │
│                 │ │ │  Convert Back to 28-bit     │ │
│                 │ │ │                             │ │
│                 │ │ │ result = convert_from_      │ │
│                 │ │ │          crypto(result_cb)  │ │
│                 │ │ └─────────────────────────────┘ │
└─────────────────┘ └─────────────────────────────────┘
        │                           │
        └─────────┬─────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│         Result Processing           │
│                                     │
│ • Set sign bit (XOR of inputs)      │
│ • Normalize result (remove zeros)   │
│ • Validate bounds                   │
└─────────────────┬───────────────────┘
                  │
                  ▼
Output: Result KcBignum
```

## Security Module Architecture

### Constant-Time Operations Design

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Security Module Architecture                     │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                      Security Principles                           │
│                                                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐ │
│  │  Constant-Time  │  │ Memory Access   │  │   Error Handling    │ │
│  │   Operations    │  │   Patterns      │  │                     │ │
│  │                 │  │                 │  │ • Uniform paths     │ │
│  │ • No branching  │  │ • Fixed access  │  │ • No information    │ │
│  │   on secrets    │  │   patterns      │  │   leakage           │ │
│  │ • Bit masking   │  │ • Cache-line    │  │ • Deterministic     │ │
│  │ • Uniform       │  │   alignment     │  │   behavior          │ │
│  │   timing        │  │                 │  │                     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                   Secure Compare Implementation                     │
│                                                                     │
│  fn secure_compare(a: &KcBignum, b: &KcBignum) -> u32 {            │
│      let mut gt = 0u32;                                             │
│      let mut eq = 1u32;                                             │
│                                                                     │
│      // Process all elements regardless of actual length           │
│      for i in (0..MAX_ELEMENTS).rev() {                            │
│          let diff = b.data[i].wrapping_sub(a.data[i]);             │
│          gt |= (diff >> 31) & eq;                                   │
│          eq &= ((b.data[i] ^ a.data[i]).wrapping_sub(1)) >> 31;    │
│      }                                                              │
│                                                                     │
│      2 + (eq - gt)  // Returns BN_BIGGER, BN_SMALLER, or BN_EQUAL  │
│  }                                                                  │
│                                                                     │
│  Key Properties:                                                    │
│  • Processes all 80 elements every time                            │
│  • No conditional branches on data                                 │
│  • Uses bit manipulation instead of comparisons                    │
│  • Timing independent of input values                              │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                     Security Validation Pipeline                   │
│                                                                     │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────────────┐   │
│  │   Static    │────▶│  Runtime    │────▶│   Side-Channel      │   │
│  │  Analysis   │     │  Testing    │     │    Testing          │   │
│  │             │     │             │     │                     │   │
│  │ • Code      │     │ • dudect    │     │ • Power analysis    │   │
│  │   review    │     │   timing    │     │ • EM analysis       │   │
│  │ • Audit     │     │   tests     │     │ • Cache timing      │   │
│  │   tools     │     │ • Property  │     │ • Micro-arch        │   │
│  │             │     │   testing   │     │   attacks           │   │
│  └─────────────┘     └─────────────┘     └─────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## Memory Layout Diagrams

### C vs Rust Memory Layout Comparison

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Memory Layout Comparison                       │
└─────────────────────────────────────────────────────────────────────┘

C Implementation:
┌─────────────────────────────────────────────────────────────────────┐
│                          Stack Frame                               │
│                                                                     │
│  KC_BIGNUM a;           KC_BIGNUM b;           KC_BIGNUM result;    │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐ │
│  │ Used: 4 bytes   │    │ Used: 4 bytes   │    │ Used: 4 bytes   │ │
│  │ Sign: 4 bytes   │    │ Sign: 4 bytes   │    │ Sign: 4 bytes   │ │
│  │ Obfus: 4 bytes  │    │ Obfus: 4 bytes  │    │ Obfus: 4 bytes  │ │
│  │ Data: 320 bytes │    │ Data: 320 bytes │    │ Data: 320 bytes │ │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘ │
│                                                                     │
│  KC_BN_SCRATCH scratch;                                             │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Barrett_u: 332 bytes                                        │   │
│  │ BarrettTemp: 996 bytes (3 × 332)                           │   │
│  │ Temp: 1992 bytes (6 × 332)                                 │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Total Stack Usage: ~4.3 KB                                        │
└─────────────────────────────────────────────────────────────────────┘

Rust Implementation (Hybrid):
┌─────────────────────────────────────────────────────────────────────┐
│                          Stack Frame                               │
│                                                                     │
│  let mut a = KcBignum::new();    // Same layout as C                │
│  ┌─────────────────┐    ┌─────────────────────────────────────────┐ │
│  │ #[repr(C)]      │    │        Additional Rust Benefits       │ │
│  │ struct KcBignum │    │                                         │ │
│  │                 │    │ • Bounds checking in debug builds      │ │
│  │ used: u32       │    │ • Automatic initialization             │ │
│  │ sign: u32       │    │ • Drop trait for cleanup              │ │
│  │ obfuscated: u32 │    │ • Ownership tracking                   │ │
│  │ data: [u32; 80] │    │ • Zero-cost abstractions              │ │
│  └─────────────────┘    └─────────────────────────────────────────┘ │
│                                                                     │
│  Temporary crypto-bigint buffers (stack-allocated):                │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ U2048 conversion buffers (when needed)                      │   │
│  │ • Only allocated during complex operations                  │   │
│  │ • Automatically dropped after use                          │   │
│  │ • No heap allocation                                        │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Total Stack Usage: ~4.3 KB + temporary buffers                    │
└─────────────────────────────────────────────────────────────────────┘
```

### 28-bit to 64-bit Conversion Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│              28-bit to 64-bit Radix Conversion                     │
└─────────────────────────────────────────────────────────────────────┘

28-bit Representation (KcBignum):
┌─────────────────────────────────────────────────────────────────────┐
│ Element │ 28-bit Value │ Bit Position │ Total Bits                  │
├─────────────────────────────────────────────────────────────────────┤
│ data[0] │ 0x0AAAAAAA  │ 0-27         │ 28 bits                     │
│ data[1] │ 0x0BBBBBBB  │ 28-55        │ 56 bits                     │
│ data[2] │ 0x0CCCCCCC  │ 56-83        │ 84 bits                     │
│ data[3] │ 0x0DDDDDDD  │ 84-111       │ 112 bits                    │
│ ...     │ ...         │ ...          │ ...                         │
└─────────────────────────────────────────────────────────────────────┘

Conversion Process:
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  fn to_crypto_bigint(&self) -> U2048 {                             │
│      let mut result = [0u64; 32]; // U2048 = 32 × 64-bit limbs     │
│      let mut bit_offset = 0;                                       │
│                                                                     │
│      for i in 0..self.used {                                       │
│          let value = self.data[i] as u64;                          │
│          let limb_index = bit_offset / 64;                         │
│          let bit_index = bit_offset % 64;                          │
│                                                                     │
│          // Pack 28-bit value into 64-bit limbs                    │
│          result[limb_index] |= value << bit_index;                 │
│          if bit_index + 28 > 64 {                                  │
│              result[limb_index + 1] |= value >> (64 - bit_index);  │
│          }                                                         │
│                                                                     │
│          bit_offset += 28;                                         │
│      }                                                             │
│                                                                     │
│      U2048::from_le_bytes(&result.as_bytes())                      │
│  }                                                                  │
└─────────────────────────────────────────────────────────────────────┘

64-bit Representation (crypto-bigint U2048):
┌─────────────────────────────────────────────────────────────────────┐
│ Limb    │ 64-bit Value         │ Bit Position │ Total Bits         │
├─────────────────────────────────────────────────────────────────────┤
│ limb[0] │ 0x1111111111111111   │ 0-63         │ 64 bits            │
│ limb[1] │ 0x2222222222222222   │ 64-127       │ 128 bits           │
│ limb[2] │ 0x3333333333333333   │ 128-191      │ 192 bits           │
│ ...     │ ...                  │ ...          │ ...                │
│ limb[31]│ 0xFFFFFFFFFFFFFFFF   │ 1984-2047    │ 2048 bits          │
└─────────────────────────────────────────────────────────────────────┘
```

## Implementation Dependencies

### Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Dependency Architecture                     │
└─────────────────────────────────────────────────────────────────────┘

SEV Firmware Application Layer
│
├── ECC Operations
│   └── KcBignum (28-bit arithmetic)
│
├── RSA Operations  
│   └── KcBignum (Barrett reduction)
│
└── AES Key Management
    └── KcBignum (modular operations)

Rust Bignum Library Dependencies:
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  Application Code                                                   │
│  ┌─────────────┐                                                    │
│  │ use bignum  │                                                    │
│  │             │                                                    │
│  │ KcBignum::  │                                                    │
│  │   new()     │                                                    │
│  │   .add()    │                                                    │
│  │   .multiply │                                                    │
│  └──────┬──────┘                                                    │
│         │                                                           │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                   Bignum Crate                              │   │
│  │                                                             │   │
│  │  ┌─────────────┐        ┌─────────────────────────────┐    │   │
│  │  │   Public    │        │      Internal Modules       │    │   │
│  │  │    API      │        │                             │    │   │
│  │  │             │        │ • arithmetic_28bit          │    │   │
│  │  │ • KcBignum  │◄──────►│ • conversion               │    │   │
│  │  │ • KcScratch │        │ • security                  │    │   │
│  │  │ • Functions │        │ • validation                │    │   │
│  │  └─────────────┘        └─────────────────────────────┘    │   │
│  │                                     │                      │   │
│  │                                     ▼                      │   │
│  │  ┌─────────────────────────────────────────────────────┐   │   │
│  │  │              External Dependencies               │   │   │
│  │  │                                                     │   │   │
│  │  │ [dependencies]                                      │   │   │
│  │  │ crypto-bigint = "0.5"                              │   │   │
│  │  │ zeroize = "1.0"  # For secure memory clearing      │   │   │
│  │  │                                                     │   │   │
│  │  │ [features]                                          │   │   │
│  │  │ default = ["std"]                                   │   │   │
│  │  │ no_std = []                                         │   │   │
│  │  │ secure_only = ["crypto-bigint/constant-time"]      │   │   │
│  │  └─────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

**Document Version**: 1.0  
**Date**: August 1, 2025  
**Authors**: AMD ASPFW Architecture Team  
**Classification**: Technical Documentation
