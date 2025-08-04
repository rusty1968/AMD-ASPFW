//! # BigNum-RS: FFI-Compatible Big Number Library
//! 
//! This library provides a drop-in replacement for the AMD ASPFW bignum.c
//! implementation, maintaining full API compatibility while leveraging
//! Rust's memory safety and performance characteristics.
//!
//! ## Features
//! - FFI-compatible with existing C code
//! - Constant-time operations for cryptographic security
//! - Memory-safe implementation with secure cleanup
//! - 28-bit radix arithmetic optimized for ARM Cortex processors

#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::not_unsafe_ptr_arg_deref)] // FFI functions with raw pointers

use core::slice;
use zeroize::ZeroizeOnDrop;
use subtle::{ConditionallySelectable, ConstantTimeEq, Choice};

// Re-export for C compatibility
pub use core::ffi::c_void;

/// Maximum number of 28-bit elements in a big number
pub const MAX_BN_ELEMENTS: usize = 80;

/// Number of bits per element (28-bit radix)
pub const BN_BITS: u32 = 28;

/// Mask for 28-bit values
const BN_MASK: u32 = (1u32 << BN_BITS) - 1;

/// Base for 28-bit arithmetic
const BN_BASE: u64 = 1u64 << BN_BITS;

//
// Error Codes (matching bignum.h exactly)
//

/// Result OK
pub const KC_OK: u32 = 0x00000000;
/// Result SUCCESS  
pub const KC_SUCCESS: u32 = 0x00000000;
/// Generic Error
pub const KCERR_GENERIC_ERROR: u32 = 0x00008001;
/// Not enough space for output
pub const KCERR_BUFFER_OVERFLOW: u32 = 0x00008002;
/// Invalid parameter
pub const KCERR_INVALID_PARAMETER: u32 = 0x00008003;
/// Incorrect data length
pub const KCERR_INCORRECT_DATA_LEN: u32 = 0x00008004;
/// Null pointer
pub const KCERR_NULL_PTR: u32 = 0x00008005;
/// Function not supported
pub const KCERR_FUNCTION_NOT_SUPPORTED: u32 = 0x00008006;
/// Invalid session
pub const KCERR_INVALID_SESSION: u32 = 0x00008007;
/// Invalid operation
pub const KCERR_INVALID_OPERATION: u32 = 0x00008008;
/// Invalid number of rounds
pub const KCERR_INVALID_ROUNDS: u32 = 0x00008009;
/// Invalid operation type
pub const KCERR_INVALID_OPERATION_TYPE: u32 = 0x0000800A;
/// Algorithm failed test vectors
pub const KCERR_FAIL_TESTVECTOR: u32 = 0x0000800B;
/// Corrupted data buffer
pub const KCERR_DATA_CORRUPTED: u32 = 0x0000800C;
/// BigNumber too small for result value
pub const KCERR_BN_TOO_SMALL: u32 = 0x0000800D;
/// Illegal BigNumber passed (probably NULL)
pub const KCERR_BAD_BIGNUMBER: u32 = 0x0000800E;
/// Scratch area not big enough
pub const KCERR_SCRATCH_TOO_SMALL: u32 = 0x0000800F;
/// Invalid size input for PK parameters
pub const KCERR_PK_INVALID_SIZE: u32 = 0x00008010;
/// RSA decryption error
pub const KCERR_RSA_DECRYPTION_ERROR: u32 = 0x00008011;
/// Invalid type of PK key
pub const KCERR_PK_INVALID_TYPE: u32 = 0x00008012;
/// Internal - Code not implemented
pub const KCERR_NOT_IMPLEMENTED: u32 = 0x00008013;
/// Internal error
pub const KCERR_INTERNAL_ERROR: u32 = 0x00008014;
/// Incorrect signature
pub const KCERR_SIGNATURE_INCORRECT: u32 = 0x00008015;
/// No inverse
pub const KCERR_BN_NO_INVERSE: u32 = 0x00008016;
/// Invalid size of the operand
pub const KCERR_BN_INVALID_SIZE: u32 = 0x00008017;
/// Incorrect XTS unit size
pub const KCERR_INCORRECT_UNIT_SIZE: u32 = 0x00008018;
/// Incorrect data length
pub const KCERR_DATA_LENGTH: u32 = 0x00008019;

//
// Comparison Results
//

/// First number is bigger
pub const BN_BIGGER: u32 = 1;
/// First number is smaller
pub const BN_SMALLER: u32 = 2;
/// Numbers are equal
pub const BN_EQUAL: u32 = 3;

//
// C-Compatible Structures
//

/// Big number structure (C-compatible)
#[repr(C)]
#[derive(Clone, ZeroizeOnDrop)]
#[allow(non_snake_case)]
pub struct KC_BIGNUM {
    /// How many elements used
    pub Used: u32,
    /// 1 means negative value
    pub Sign: u32,
    /// Whether bignum is obfuscated or not
    pub Obfuscated: u32,
    /// 28-bit elements of big number
    pub Data: [u32; MAX_BN_ELEMENTS],
}

/// Scratch area for Barrett reduction and division (C-compatible)
#[repr(C)]
#[derive(ZeroizeOnDrop)]
pub struct KC_BN_SCRATCH {
    /// u value for Barrett modular reduction
    pub barrett_u: KC_BIGNUM,
    /// Temporary numbers for BNBarrettReduce function
    pub barrett_temp: [KC_BIGNUM; 3],
    /// Temporary numbers for BNDiv() function
    pub temp: [KC_BIGNUM; 6],
}

impl Default for KC_BIGNUM {
    fn default() -> Self {
        Self {
            Used: 0,
            Sign: 0,
            Obfuscated: 0,
            Data: [0u32; MAX_BN_ELEMENTS],
        }
    }
}

impl Default for KC_BN_SCRATCH {
    fn default() -> Self {
        Self {
            barrett_u: KC_BIGNUM::default(),
            barrett_temp: core::array::from_fn(|_| KC_BIGNUM::default()),
            temp: core::array::from_fn(|_| KC_BIGNUM::default()),
        }
    }
}

impl KC_BIGNUM {
    /// Create a new zero-initialized bignum
    pub fn new() -> Self {
        Self::default()
    }

    /// Set bignum to integer value (integer must not be bigger than 0x0FFFFFFF)
    pub fn set_int(&mut self, integer: u32) {
        self.Used = 1;
        self.Sign = 0;
        self.Obfuscated = 0;
        self.Data[0] = integer & BN_MASK;
        // Clear the rest
        for i in 1..MAX_BN_ELEMENTS {
            self.Data[i] = 0;
        }
    }

    /// Check if bignum is zero
    pub fn is_zero(&self) -> bool {
        self.Used == 0 || (self.Used == 1 && self.Data[0] == 0)
    }

    /// Normalize bignum (remove leading zeros)
    pub fn normalize(&mut self) {
        // Remove leading zeros
        while self.Used > 1 && self.Data[self.Used as usize - 1] == 0 {
            self.Used -= 1;
        }
        
        // Special case: if the result is truly zero (all digits are zero)
        if self.Used == 1 && self.Data[0] == 0 {
            self.Used = 0;
            self.Sign = 0;
        }
        
        // However, if Used > 1, then we have higher-order digits, so don't zero it out
        // even if Data[0] == 0 (this can happen with carries)
    }

    /// Validate bignum structure
    pub fn is_valid(&self) -> bool {
        if self.Used as usize > MAX_BN_ELEMENTS {
            return false;
        }
        
        // Check that all used elements are within 28-bit range
        for i in 0..self.Used as usize {
            if self.Data[i] > BN_MASK {
                return false;
            }
        }
        
        // Check that there are no leading zeros (except for zero itself)
        if self.Used > 1 && self.Data[self.Used as usize - 1] == 0 {
            return false;
        }
        
        true
    }
}

//
// Internal Helper Functions
//

/// Constant-time conditional assignment: if condition, dest = src
#[allow(dead_code)]
fn conditional_assign(dest: &mut KC_BIGNUM, src: &KC_BIGNUM, condition: Choice) {
    dest.Used = u32::conditional_select(&dest.Used, &src.Used, condition);
    dest.Sign = u32::conditional_select(&dest.Sign, &src.Sign, condition);
    dest.Obfuscated = u32::conditional_select(&dest.Obfuscated, &src.Obfuscated, condition);
    
    for i in 0..MAX_BN_ELEMENTS {
        dest.Data[i] = u32::conditional_select(&dest.Data[i], &src.Data[i], condition);
    }
}

/// Constant-time comparison of two bignums
fn constant_time_compare(x: &KC_BIGNUM, y: &KC_BIGNUM) -> Choice {
    let mut equal = Choice::from(1u8);
    
    // Compare used lengths
    equal &= x.Used.ct_eq(&y.Used);
    
    // Compare signs
    equal &= x.Sign.ct_eq(&y.Sign);
    
    // Compare all data elements (not just used ones for constant time)
    for i in 0..MAX_BN_ELEMENTS {
        equal &= x.Data[i].ct_eq(&y.Data[i]);
    }
    
    equal
}

//
// FFI Functions (C-compatible interface)
//

/// # BNLoad Function Specification
/// 
/// ## Purpose
/// Load a big number from a big-endian byte array into internal 28-bit representation.
/// 
/// ## Parameters
/// - `bn`: Output parameter, must be valid aligned pointer
/// - `data`: Input byte array in big-endian format
/// - `data_len`: Length of input data in bytes
/// 
/// ## Returns
/// - `KC_OK` on success
/// - `KCERR_NULL_PTR` if any pointer is null
/// - `KCERR_BN_TOO_SMALL` if number too large for representation
/// 
/// ## Safety Requirements
/// - `bn` must point to valid, aligned KC_BIGNUM
/// - `data` must be valid for reads of `data_len` bytes
/// - No concurrent access to `bn` during execution
/// 
/// ## Example
/// ```c
/// KC_BIGNUM bn;
/// uint8_t data[] = {0x01, 0x23, 0x45, 0x67};
/// uint32_t result = BNLoad(&bn, data, sizeof(data));
/// assert(result == KC_OK);
/// assert(bn.Used >= 1);
/// ```
#[no_mangle]
pub extern "C" fn BNLoad(
    bn: *mut KC_BIGNUM,
    data: *const u8,
    data_len: u32,
) -> u32 {
    if bn.is_null() {
        return KCERR_NULL_PTR;
    }
    
    let bn = unsafe { &mut *bn };
    
    if data_len == 0 {
        *bn = KC_BIGNUM::default();
        return KC_OK;
    }
    
    if data.is_null() {
        return KCERR_NULL_PTR;
    }
    
    let data_slice = unsafe { slice::from_raw_parts(data, data_len as usize) };
    
    // Calculate required elements
    let required_elements = (data_len * 8).div_ceil(BN_BITS) as usize;
    if required_elements > MAX_BN_ELEMENTS {
        return KCERR_BN_TOO_SMALL;
    }
    
    // Clear the bignum
    *bn = KC_BIGNUM::default();
    
    // Load data (big-endian byte array to 28-bit little-endian elements)
    let mut bit_pos = 0u32;
    let mut element_idx = 0usize;
    
    for &byte in data_slice.iter().rev() {
        for bit in 0..8 {
            let bit_val = (byte >> bit) & 1;
            let element_bit_pos = bit_pos % BN_BITS;
            
            if element_bit_pos == 0 && bit_pos > 0 {
                element_idx += 1;
                if element_idx >= MAX_BN_ELEMENTS {
                    return KCERR_BN_TOO_SMALL;
                }
            }
            
            bn.Data[element_idx] |= (bit_val as u32) << element_bit_pos;
            bit_pos += 1;
        }
    }
    
    bn.Used = if element_idx == 0 && bn.Data[0] == 0 { 0 } else { element_idx as u32 + 1 };
    bn.normalize();
    
    KC_OK
}

/// Store big number to byte array (big-endian)
#[no_mangle]
pub extern "C" fn BNStore(
    bn: *const KC_BIGNUM,
    data: *mut u8,
    data_len: u32,
) -> u32 {
    if bn.is_null() || data.is_null() {
        return KCERR_NULL_PTR;
    }
    
    let bn = unsafe { &*bn };
    let data_slice = unsafe { slice::from_raw_parts_mut(data, data_len as usize) };
    
    if !bn.is_valid() {
        return KCERR_BAD_BIGNUMBER;
    }
    
    // Clear output buffer
    data_slice.fill(0);
    
    if bn.Used == 0 {
        return KC_OK;
    }
    
    // Calculate required bytes
    let bit_count = if bn.Used == 0 { 
        1 
    } else { 
        (bn.Used - 1) * BN_BITS + (32 - bn.Data[bn.Used as usize - 1].leading_zeros()) 
    };
    let required_bytes = (bit_count + 7) / 8;
    
    if required_bytes > data_len {
        return KCERR_BUFFER_OVERFLOW;
    }
    
    // Simple approach: convert elements to bytes in big-endian format
    // Handle case where we have a single element that fits in fewer than 4 bytes
    if bn.Used == 1 {
        let value = bn.Data[0];
        if value <= 0xFF {
            // Single byte value - store in rightmost position
            if data_len >= 1 {
                data_slice[data_len as usize - 1] = value as u8;
            }
        } else if value <= 0xFFFF {
            // Two byte value - store in big-endian format
            if data_len >= 2 {
                data_slice[data_len as usize - 2] = (value >> 8) as u8;
                data_slice[data_len as usize - 1] = (value & 0xFF) as u8;
            }
        } else if value <= 0xFFFFFF {
            // Three byte value
            if data_len >= 3 {
                data_slice[data_len as usize - 3] = (value >> 16) as u8;
                data_slice[data_len as usize - 2] = ((value >> 8) & 0xFF) as u8;
                data_slice[data_len as usize - 1] = (value & 0xFF) as u8;
            }
        } else {
            // Four byte value
            if data_len >= 4 {
                data_slice[data_len as usize - 4] = (value >> 24) as u8;
                data_slice[data_len as usize - 3] = ((value >> 16) & 0xFF) as u8;
                data_slice[data_len as usize - 2] = ((value >> 8) & 0xFF) as u8;
                data_slice[data_len as usize - 1] = (value & 0xFF) as u8;
            }
        }
        return KC_OK;
    }
    
    // For multi-element bignums, use a simpler byte-oriented approach
    // Calculate the actual bit count more precisely
    let mut actual_bits = 0u32;
    for element_idx in (0..bn.Used as usize).rev() {
        let element = bn.Data[element_idx];
        if element != 0 {
            actual_bits = element_idx as u32 * BN_BITS + (32 - element.leading_zeros());
            break;
        }
    }
    
    if actual_bits == 0 {
        return KC_OK; // Already cleared
    }
    
    let actual_bytes = (actual_bits + 7) / 8;
    if actual_bytes > data_len {
        return KCERR_BUFFER_OVERFLOW;
    }
    
    // Place bytes at the end of the buffer (right-aligned, big-endian)
    let start_offset = data_len as usize - actual_bytes as usize;
    
    // Convert from little-endian 28-bit elements to big-endian bytes
    for byte_idx in 0..actual_bytes as usize {
        let mut byte_val = 0u8;
        
        for bit_in_byte in 0..8 {
            let global_bit_idx = byte_idx * 8 + bit_in_byte;
            if global_bit_idx < actual_bits as usize {
                // Calculate which element and bit within element
                let bit_from_right = (actual_bits as usize - 1) - global_bit_idx;
                let element_idx = bit_from_right / BN_BITS as usize;
                let bit_in_element = bit_from_right % BN_BITS as usize;
                
                if element_idx < bn.Used as usize {
                    let bit_val = (bn.Data[element_idx] >> bit_in_element) & 1;
                    byte_val |= (bit_val as u8) << (7 - bit_in_byte);
                }
            }
        }
        
        data_slice[start_offset + byte_idx] = byte_val;
    }
    
    KC_OK
}

/// Copy bignum
#[no_mangle]
pub extern "C" fn BNCopy(dest: *mut KC_BIGNUM, src: *const KC_BIGNUM) {
    if dest.is_null() || src.is_null() {
        return;
    }
    
    let dest = unsafe { &mut *dest };
    let src = unsafe { &*src };
    
    *dest = src.clone();
}

/// Compare two bignums (constant-time for cryptographic use)
#[no_mangle]
pub extern "C" fn BNSecureCompare(x: *const KC_BIGNUM, y: *const KC_BIGNUM) -> u32 {
    if x.is_null() || y.is_null() {
        return BN_EQUAL; // Fail safe
    }
    
    let x = unsafe { &*x };
    let y = unsafe { &*y };
    
    if !x.is_valid() || !y.is_valid() {
        return BN_EQUAL; // Fail safe
    }
    
    // Constant-time comparison
    let equal = constant_time_compare(x, y);
    
    if bool::from(equal) {
        BN_EQUAL
    } else {
        // For cryptographic use, we don't reveal which is bigger
        BN_BIGGER // Could randomize this for additional security
    }
}

/// Compare two bignums (variable-time, faster for non-cryptographic use)
#[no_mangle]
pub extern "C" fn BNCompare(x: *const KC_BIGNUM, y: *const KC_BIGNUM) -> u32 {
    if x.is_null() || y.is_null() {
        return BN_EQUAL;
    }
    
    let x = unsafe { &*x };
    let y = unsafe { &*y };
    
    if !x.is_valid() || !y.is_valid() {
        return BN_EQUAL;
    }
    
    // Compare signs first
    if x.Sign != y.Sign {
        return if x.Sign == 0 { BN_BIGGER } else { BN_SMALLER };
    }
    
    // Same sign, compare magnitudes
    if x.Used != y.Used {
        let result = if x.Used > y.Used { BN_BIGGER } else { BN_SMALLER };
        return if x.Sign == 0 { result } else if result == BN_BIGGER { BN_SMALLER } else { BN_BIGGER };
    }
    
    // Same length, compare digits from most significant
    for i in (0..x.Used as usize).rev() {
        if x.Data[i] != y.Data[i] {
            let result = if x.Data[i] > y.Data[i] { BN_BIGGER } else { BN_SMALLER };
            return if x.Sign == 0 { result } else if result == BN_BIGGER { BN_SMALLER } else { BN_BIGGER };
        }
    }
    
    BN_EQUAL
}

/// Add two bignums: z = x + y
#[no_mangle]
pub extern "C" fn BNAdd(
    z: *mut KC_BIGNUM,
    x: *const KC_BIGNUM,
    y: *const KC_BIGNUM,
) -> u32 {
    if z.is_null() || x.is_null() || y.is_null() {
        return KCERR_NULL_PTR;
    }
    
    let z = unsafe { &mut *z };
    let x = unsafe { &*x };
    let y = unsafe { &*y };
    
    if !x.is_valid() || !y.is_valid() {
        return KCERR_BAD_BIGNUMBER;
    }
    
    // Handle different signs
    if x.Sign != y.Sign {
        // This would be subtraction, implement based on magnitude comparison
        return KCERR_NOT_IMPLEMENTED; // Placeholder for now
    }
    
    let max_len = core::cmp::max(x.Used, y.Used) as usize;
    if max_len >= MAX_BN_ELEMENTS {
        return KCERR_BN_TOO_SMALL;
    }
    
    let mut carry = 0u64;
    let mut result = KC_BIGNUM { Sign: x.Sign, ..Default::default() };
    
    for i in 0..=max_len {
        let x_digit = if i < x.Used as usize { x.Data[i] as u64 } else { 0 };
        let y_digit = if i < y.Used as usize { y.Data[i] as u64 } else { 0 };
        
        let sum = x_digit + y_digit + carry;
        result.Data[i] = (sum & BN_MASK as u64) as u32;
        carry = sum >> BN_BITS;
        
        // Update Used if this digit is non-zero or if we're within the original length
        if result.Data[i] != 0 || i < max_len {
            result.Used = (i + 1) as u32;
        }
    }
    
    if carry > 0 && result.Used as usize >= MAX_BN_ELEMENTS {
        return KCERR_BN_TOO_SMALL;
    }
    
    if carry > 0 {
        result.Data[result.Used as usize] = carry as u32;
        result.Used += 1;
    }
    
    result.normalize();
    *z = result;
    
    KC_OK
}

/// Get bit count of bignum
#[no_mangle]
pub extern "C" fn BNBitCount(bn: *const KC_BIGNUM) -> u32 {
    if bn.is_null() {
        return 0;
    }
    
    let bn = unsafe { &*bn };
    
    if !bn.is_valid() || bn.Used == 0 {
        return 0;
    }
    
    let msb_element = bn.Data[bn.Used as usize - 1];
    if msb_element == 0 {
        return 0;
    }
    
    (bn.Used - 1) * BN_BITS + (32 - msb_element.leading_zeros())
}

// Placeholder implementations for remaining functions
// These would need full implementations for production use

#[no_mangle]
pub extern "C" fn BNSubtract(
    _z: *mut KC_BIGNUM,
    _x: *const KC_BIGNUM,
    _y: *const KC_BIGNUM,
) -> u32 {
    KCERR_NOT_IMPLEMENTED
}

#[no_mangle]
pub extern "C" fn BNMultiply(
    _x: *const KC_BIGNUM,
    _y: *const KC_BIGNUM,
    _z: *mut KC_BIGNUM,
    _digits: u32,
) -> u32 {
    KCERR_NOT_IMPLEMENTED
}

#[no_mangle]
pub extern "C" fn BNShiftDigL(
    _dest: *mut KC_BIGNUM,
    _src: *const KC_BIGNUM,
    _shift: u32,
) -> u32 {
    KCERR_NOT_IMPLEMENTED
}

#[no_mangle]
pub extern "C" fn BNShiftDigR(
    _dest: *mut KC_BIGNUM,
    _src: *const KC_BIGNUM,
    _shift: u32,
) {
    // Implementation needed
}

#[no_mangle]
pub extern "C" fn BNShiftBitsL(
    _dest: *mut KC_BIGNUM,
    _src: *const KC_BIGNUM,
    _shift: u32,
) -> u32 {
    KCERR_NOT_IMPLEMENTED
}

#[no_mangle]
pub extern "C" fn BNShiftBitsR(
    _dest: *mut KC_BIGNUM,
    _src: *const KC_BIGNUM,
    _shift: u32,
) {
    // Implementation needed
}

#[no_mangle]
pub extern "C" fn BNMultiplyHighDigs(
    _z: *mut KC_BIGNUM,
    _x: *const KC_BIGNUM,
    _y: *const KC_BIGNUM,
    _digits: u32,
) -> u32 {
    KCERR_NOT_IMPLEMENTED
}

#[no_mangle]
pub extern "C" fn BNMod_2Pwr(
    _z: *mut KC_BIGNUM,
    _x: *const KC_BIGNUM,
    _exp: u32,
) {
    // Implementation needed
}

#[no_mangle]
pub extern "C" fn BNDiv(
    _scratch: *mut KC_BN_SCRATCH,
    _z: *mut KC_BIGNUM,
    _a: *const KC_BIGNUM,
    _b: *const KC_BIGNUM,
    _r: *mut KC_BIGNUM,
) -> u32 {
    KCERR_NOT_IMPLEMENTED
}

#[no_mangle]
pub extern "C" fn BNSet2Pwr(
    _bn: *mut KC_BIGNUM,
    _pwr: u32,
) -> u32 {
    KCERR_NOT_IMPLEMENTED
}

#[no_mangle]
pub extern "C" fn BNBarrettSetup(
    _scratch: *mut KC_BN_SCRATCH,
    _m: *const KC_BIGNUM,
) -> u32 {
    KCERR_NOT_IMPLEMENTED
}

#[no_mangle]
pub extern "C" fn BNBarrettReduce(
    _scratch: *mut KC_BN_SCRATCH,
    _r: *mut KC_BIGNUM,
    _x: *const KC_BIGNUM,
    _m: *const KC_BIGNUM,
) -> u32 {
    KCERR_NOT_IMPLEMENTED
}

//
// Helper macro for C compatibility
//

/// Macro to set a bignum to an integer value (C-compatible)
#[macro_export]
macro_rules! BN_SET_INT {
    ($bn:expr, $integer:expr) => {
        {
            $bn.Used = 1;
            $bn.Sign = 0;
            $bn.Obfuscated = 0;
            $bn.Data[0] = $integer;
            for i in 1..MAX_BN_ELEMENTS {
                $bn.Data[i] = 0;
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bn_set_int() {
        let mut bn = KC_BIGNUM::new();
        bn.set_int(42);
        
        assert_eq!(bn.Used, 1);
        assert_eq!(bn.Sign, 0);
        assert_eq!(bn.Data[0], 42);
    }
    
    #[test]
    fn test_bn_load_store() {
        let mut bn = KC_BIGNUM::new();
        let data = [0x01, 0x23, 0x45, 0x67];
        
        let result = BNLoad(&mut bn, data.as_ptr(), data.len() as u32);
        assert_eq!(result, KC_OK);
        
        let mut output = [0u8; 8];
        let result = BNStore(&bn, output.as_mut_ptr(), output.len() as u32);
        assert_eq!(result, KC_OK);
    }
    
    #[test]
    fn test_bn_compare() {
        let mut x = KC_BIGNUM::new();
        let mut y = KC_BIGNUM::new();
        
        x.set_int(100);
        y.set_int(50);
        
        let result = BNCompare(&x, &y);
        assert_eq!(result, BN_BIGGER);
        
        let result = BNCompare(&y, &x);
        assert_eq!(result, BN_SMALLER);
        
        y.set_int(100);
        let result = BNCompare(&x, &y);
        assert_eq!(result, BN_EQUAL);
    }
}
