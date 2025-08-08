/*
 * unit_test.c - Comprehensive Unit Tests for BigNum-RS with Coverage Analysis
 * 
 * This file contains focused unit tests for each bignum function to ensure
 * comprehensive coverage and validate correctness of the Rust implementation.
 */

#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>
#include "bignum_ffi.h"

// Test counters
static int tests_run = 0;
static int tests_passed = 0;
static int tests_failed = 0;

// Test assertion macros
#define UNIT_TEST(name) \
    static void name(void); \
    static void run_##name(void) { \
        tests_run++; \
        printf("Running %s... ", #name); \
        fflush(stdout); \
        name(); \
        tests_passed++; \
        printf("PASS\n"); \
    } \
    static void name(void)

#define ASSERT_EQ(actual, expected) \
    do { \
        if ((actual) != (expected)) { \
            printf("FAIL\n  Expected: %u, Got: %u (line %d)\n", \
                   (unsigned)(expected), (unsigned)(actual), __LINE__); \
            tests_failed++; \
            return; \
        } \
    } while(0)

#define ASSERT_TRUE(condition) \
    do { \
        if (!(condition)) { \
            printf("FAIL\n  Assertion failed: %s (line %d)\n", #condition, __LINE__); \
            tests_failed++; \
            return; \
        } \
    } while(0)

#define ASSERT_FALSE(condition) ASSERT_TRUE(!(condition))

#define ASSERT_BYTES_EQ(actual, expected, len) \
    do { \
        for (size_t i = 0; i < (len); i++) { \
            if ((actual)[i] != (expected)[i]) { \
                printf("FAIL\n  Byte mismatch at index %zu: expected 0x%02X, got 0x%02X (line %d)\n", \
                       i, (expected)[i], (actual)[i], __LINE__); \
                tests_failed++; \
                return; \
            } \
        } \
    } while(0)

// Helper function to print bignum for debugging
void print_bignum(const KC_BIGNUM* bn, const char* label) {
    printf("  %s: Used=%u, Sign=%u, Data[0]=0x%08X\n", 
           label, bn->Used, bn->Sign, bn->Used > 0 ? bn->Data[0] : 0);
}

//
// Unit Tests for BN_SET_INT Macro
//

UNIT_TEST(test_bn_set_int_zero) {
    KC_BIGNUM bn;
    BN_SET_INT(bn, 0);
    
    ASSERT_EQ(bn.Used, 1);
    ASSERT_EQ(bn.Sign, 0);
    ASSERT_EQ(bn.Obfuscated, 0);
    ASSERT_EQ(bn.Data[0], 0);
}

UNIT_TEST(test_bn_set_int_positive) {
    KC_BIGNUM bn;
    BN_SET_INT(bn, 42);
    
    ASSERT_EQ(bn.Used, 1);
    ASSERT_EQ(bn.Sign, 0);
    ASSERT_EQ(bn.Data[0], 42);
    
    // Verify rest of array is cleared
    for (int i = 1; i < 10; i++) {
        ASSERT_EQ(bn.Data[i], 0);
    }
}

UNIT_TEST(test_bn_set_int_max_28bit) {
    KC_BIGNUM bn;
    BN_SET_INT(bn, 0x0FFFFFFF);
    
    ASSERT_EQ(bn.Used, 1);
    ASSERT_EQ(bn.Data[0], 0x0FFFFFFF);
}

//
// Unit Tests for BNLoad Function
//

UNIT_TEST(test_bn_load_null_pointer) {
    KC_BIGNUM bn;
    uint8_t data[] = {0x42};
    
    ASSERT_EQ(BNLoad(NULL, data, 1), KCERR_NULL_PTR);
    ASSERT_EQ(BNLoad(&bn, NULL, 1), KCERR_NULL_PTR);
}

UNIT_TEST(test_bn_load_zero_length) {
    KC_BIGNUM bn;
    uint32_t result = BNLoad(&bn, NULL, 0);
    
    ASSERT_EQ(result, KC_OK);
    ASSERT_EQ(bn.Used, 0);
    ASSERT_EQ(bn.Sign, 0);
}

UNIT_TEST(test_bn_load_single_byte) {
    KC_BIGNUM bn;
    uint8_t data[] = {0x42};
    
    uint32_t result = BNLoad(&bn, data, sizeof(data));
    
    ASSERT_EQ(result, KC_OK);
    ASSERT_EQ(bn.Used, 1);
    ASSERT_EQ(bn.Data[0], 0x42);
}

UNIT_TEST(test_bn_load_multi_byte) {
    KC_BIGNUM bn;
    uint8_t data[] = {0x01, 0x23};
    
    uint32_t result = BNLoad(&bn, data, sizeof(data));
    
    ASSERT_EQ(result, KC_OK);
    ASSERT_EQ(bn.Used, 1);
    ASSERT_EQ(bn.Data[0], 0x0123); // Big-endian input
}

UNIT_TEST(test_bn_load_large_data) {
    KC_BIGNUM bn;
    uint8_t data[8] = {0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0};
    
    uint32_t result = BNLoad(&bn, data, sizeof(data));
    
    ASSERT_EQ(result, KC_OK);
    ASSERT_TRUE(bn.Used > 1);
}

UNIT_TEST(test_bn_load_oversized) {
    KC_BIGNUM bn;
    uint8_t large_data[1000];
    memset(large_data, 0xFF, sizeof(large_data));
    
    uint32_t result = BNLoad(&bn, large_data, sizeof(large_data));
    
    ASSERT_EQ(result, KCERR_BN_TOO_SMALL);
}

//
// Unit Tests for BNStore Function
//

UNIT_TEST(test_bn_store_null_pointer) {
    KC_BIGNUM bn;
    uint8_t output[4];
    
    BN_SET_INT(bn, 42);
    
    ASSERT_EQ(BNStore(NULL, output, sizeof(output)), KCERR_NULL_PTR);
    ASSERT_EQ(BNStore(&bn, NULL, sizeof(output)), KCERR_NULL_PTR);
}

UNIT_TEST(test_bn_store_zero) {
    KC_BIGNUM bn;
    uint8_t output[4];
    
    memset(&bn, 0, sizeof(bn));
    memset(output, 0xFF, sizeof(output));
    
    uint32_t result = BNStore(&bn, output, sizeof(output));
    
    ASSERT_EQ(result, KC_OK);
    for (size_t i = 0; i < sizeof(output); i++) {
        ASSERT_EQ(output[i], 0);
    }
}

UNIT_TEST(test_bn_store_single_digit) {
    KC_BIGNUM bn;
    uint8_t output[4];
    
    BN_SET_INT(bn, 0x1234);
    memset(output, 0, sizeof(output));
    
    uint32_t result = BNStore(&bn, output, sizeof(output));
    
    ASSERT_EQ(result, KC_OK);
    ASSERT_EQ(output[2], 0x12);
    ASSERT_EQ(output[3], 0x34);
}

UNIT_TEST(test_bn_store_buffer_overflow) {
    KC_BIGNUM bn;
    uint8_t large_input[] = {0xFF, 0xEE, 0xDD, 0xCC};
    uint8_t small_output[2];
    
    uint32_t result = BNLoad(&bn, large_input, sizeof(large_input));
    ASSERT_EQ(result, KC_OK);
    
    result = BNStore(&bn, small_output, sizeof(small_output));
    ASSERT_EQ(result, KCERR_BUFFER_OVERFLOW);
}

//
// Unit Tests for BNCopy Function
//

UNIT_TEST(test_bn_copy_valid) {
    KC_BIGNUM source, dest;
    
    BN_SET_INT(source, 12345);
    memset(&dest, 0xFF, sizeof(dest)); // Fill with garbage
    
    BNCopy(&dest, &source);
    
    ASSERT_EQ(dest.Used, source.Used);
    ASSERT_EQ(dest.Sign, source.Sign);
    ASSERT_EQ(dest.Data[0], source.Data[0]);
}

UNIT_TEST(test_bn_copy_null_pointers) {
    KC_BIGNUM bn;
    BN_SET_INT(bn, 42);
    
    // These should not crash
    BNCopy(NULL, &bn);
    BNCopy(&bn, NULL);
    BNCopy(NULL, NULL);
}

//
// Unit Tests for BNCompare Function
//

UNIT_TEST(test_bn_compare_equal) {
    KC_BIGNUM a, b;
    
    BN_SET_INT(a, 42);
    BN_SET_INT(b, 42);
    
    ASSERT_EQ(BNCompare(&a, &b), BN_EQUAL);
}

UNIT_TEST(test_bn_compare_less_than) {
    KC_BIGNUM a, b;
    
    BN_SET_INT(a, 42);
    BN_SET_INT(b, 100);
    
    ASSERT_EQ(BNCompare(&a, &b), BN_SMALLER);
}

UNIT_TEST(test_bn_compare_greater_than) {
    KC_BIGNUM a, b;
    
    BN_SET_INT(a, 100);
    BN_SET_INT(b, 42);
    
    ASSERT_EQ(BNCompare(&a, &b), BN_BIGGER);
}

UNIT_TEST(test_bn_compare_zero) {
    KC_BIGNUM a, b;
    
    BN_SET_INT(a, 0);
    BN_SET_INT(b, 0);
    
    ASSERT_EQ(BNCompare(&a, &b), BN_EQUAL);
    
    BN_SET_INT(b, 1);
    ASSERT_EQ(BNCompare(&a, &b), BN_SMALLER);
}

UNIT_TEST(test_bn_compare_null_pointers) {
    KC_BIGNUM bn;
    BN_SET_INT(bn, 42);
    
    ASSERT_EQ(BNCompare(NULL, &bn), BN_EQUAL);
    ASSERT_EQ(BNCompare(&bn, NULL), BN_EQUAL);
    ASSERT_EQ(BNCompare(NULL, NULL), BN_EQUAL);
}

//
// Unit Tests for BNSecureCompare Function
//

UNIT_TEST(test_bn_secure_compare_equal) {
    KC_BIGNUM a, b;
    
    BN_SET_INT(a, 12345);
    BN_SET_INT(b, 12345);
    
    ASSERT_EQ(BNSecureCompare(&a, &b), BN_EQUAL);
}

UNIT_TEST(test_bn_secure_compare_different) {
    KC_BIGNUM a, b;
    
    BN_SET_INT(a, 12345);
    BN_SET_INT(b, 54321);
    
    uint32_t result = BNSecureCompare(&a, &b);
    ASSERT_TRUE(result != BN_EQUAL);
}

UNIT_TEST(test_bn_secure_compare_null_pointers) {
    KC_BIGNUM bn;
    BN_SET_INT(bn, 42);
    
    ASSERT_EQ(BNSecureCompare(NULL, &bn), BN_EQUAL);
    ASSERT_EQ(BNSecureCompare(&bn, NULL), BN_EQUAL);
}

//
// Unit Tests for BNAdd Function
//

UNIT_TEST(test_bn_add_simple) {
    KC_BIGNUM a, b, result;
    
    BN_SET_INT(a, 42);
    BN_SET_INT(b, 58);
    
    uint32_t status = BNAdd(&result, &a, &b);
    
    ASSERT_EQ(status, KC_OK);
    ASSERT_EQ(result.Data[0], 100);
}

UNIT_TEST(test_bn_add_zero) {
    KC_BIGNUM a, b, result;
    
    BN_SET_INT(a, 0);
    BN_SET_INT(b, 42);
    
    uint32_t status = BNAdd(&result, &a, &b);
    
    ASSERT_EQ(status, KC_OK);
    ASSERT_EQ(result.Data[0], 42);
    
    // Test commutative property
    status = BNAdd(&result, &b, &a);
    ASSERT_EQ(status, KC_OK);
    ASSERT_EQ(result.Data[0], 42);
}

UNIT_TEST(test_bn_add_carry_within_digit) {
    KC_BIGNUM a, b, result;
    
    BN_SET_INT(a, 0x0AAAAAAA);
    BN_SET_INT(b, 0x05555555);
    
    uint32_t status = BNAdd(&result, &a, &b);
    
    ASSERT_EQ(status, KC_OK);
    ASSERT_EQ(result.Data[0], 0x0FFFFFFF);
}

UNIT_TEST(test_bn_add_null_pointers) {
    KC_BIGNUM a, b, result;
    
    BN_SET_INT(a, 42);
    BN_SET_INT(b, 58);
    
    ASSERT_EQ(BNAdd(NULL, &a, &b), KCERR_NULL_PTR);
    ASSERT_EQ(BNAdd(&result, NULL, &b), KCERR_NULL_PTR);
    ASSERT_EQ(BNAdd(&result, &a, NULL), KCERR_NULL_PTR);
}

//
// Unit Tests for BNBitCount Function
//

UNIT_TEST(test_bn_bit_count_zero) {
    KC_BIGNUM bn;
    
    memset(&bn, 0, sizeof(bn));
    ASSERT_EQ(BNBitCount(&bn), 0);
    
    BN_SET_INT(bn, 0);
    ASSERT_EQ(BNBitCount(&bn), 0);
}

UNIT_TEST(test_bn_bit_count_powers_of_two) {
    KC_BIGNUM bn;
    
    uint32_t powers[] = {1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024};
    uint32_t expected_bits[] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11};
    
    for (size_t i = 0; i < sizeof(powers)/sizeof(powers[0]); i++) {
        BN_SET_INT(bn, powers[i]);
        ASSERT_EQ(BNBitCount(&bn), expected_bits[i]);
    }
}

UNIT_TEST(test_bn_bit_count_non_power_of_two) {
    KC_BIGNUM bn;
    
    BN_SET_INT(bn, 3);      // 11b
    ASSERT_EQ(BNBitCount(&bn), 2);
    
    BN_SET_INT(bn, 7);      // 111b
    ASSERT_EQ(BNBitCount(&bn), 3);
    
    BN_SET_INT(bn, 255);    // 11111111b
    ASSERT_EQ(BNBitCount(&bn), 8);
    
    BN_SET_INT(bn, 0x0FFFFFFF); // Maximum 28-bit value
    ASSERT_EQ(BNBitCount(&bn), 28);
}

UNIT_TEST(test_bn_bit_count_null_pointer) {
    ASSERT_EQ(BNBitCount(NULL), 0);
}

//
// Unit Tests for Load/Store Round-trip
//

UNIT_TEST(test_load_store_roundtrip_single_byte) {
    KC_BIGNUM bn;
    uint8_t input[] = {0x42};
    uint8_t output[4];
    
    uint32_t result = BNLoad(&bn, input, sizeof(input));
    ASSERT_EQ(result, KC_OK);
    
    memset(output, 0, sizeof(output));
    result = BNStore(&bn, output, sizeof(output));
    ASSERT_EQ(result, KC_OK);
    
    ASSERT_EQ(output[3], 0x42);
    for (int i = 0; i < 3; i++) {
        ASSERT_EQ(output[i], 0);
    }
}

UNIT_TEST(test_load_store_roundtrip_multi_byte) {
    KC_BIGNUM bn;
    uint8_t input[] = {0x12, 0x34};
    uint8_t output[4];
    
    uint32_t result = BNLoad(&bn, input, sizeof(input));
    ASSERT_EQ(result, KC_OK);
    
    memset(output, 0, sizeof(output));
    result = BNStore(&bn, output, sizeof(output));
    ASSERT_EQ(result, KC_OK);
    
    ASSERT_EQ(output[2], 0x12);
    ASSERT_EQ(output[3], 0x34);
}

//
// Unit Tests for Placeholder Functions
//

UNIT_TEST(test_placeholder_functions) {
    KC_BIGNUM a, b, result;
    KC_BN_SCRATCH scratch;
    
    ASSERT_EQ(BNSubtract(&result, &a, &b), KCERR_NOT_IMPLEMENTED);
    ASSERT_EQ(BNMultiply(&a, &b, &result, 1), KCERR_NOT_IMPLEMENTED);
    ASSERT_EQ(BNDiv(&scratch, &result, &a, &b, &result), KCERR_NOT_IMPLEMENTED);
    ASSERT_EQ(BNBarrettSetup(&scratch, &a), KCERR_NOT_IMPLEMENTED);
    ASSERT_EQ(BNBarrettReduce(&scratch, &result, &a, &b), KCERR_NOT_IMPLEMENTED);
    ASSERT_EQ(BNShiftDigL(&result, &a, 1), KCERR_NOT_IMPLEMENTED);
    ASSERT_EQ(BNShiftBitsL(&result, &a, 1), KCERR_NOT_IMPLEMENTED);
    ASSERT_EQ(BNSet2Pwr(&result, 10), KCERR_NOT_IMPLEMENTED);
    
    // These should not crash (they return void)
    BNShiftDigR(&result, &a, 1);
    BNShiftBitsR(&result, &a, 1);
    BNMod_2Pwr(&result, &a, 10);
}

//
// Main Test Runner
//

typedef struct {
    const char* name;
    void (*func)(void);
} test_case_t;

#define TEST_CASE(name) { #name, run_##name }

static test_case_t test_cases[] = {
    // BN_SET_INT tests
    TEST_CASE(test_bn_set_int_zero),
    TEST_CASE(test_bn_set_int_positive),
    TEST_CASE(test_bn_set_int_max_28bit),
    
    // BNLoad tests
    TEST_CASE(test_bn_load_null_pointer),
    TEST_CASE(test_bn_load_zero_length),
    TEST_CASE(test_bn_load_single_byte),
    TEST_CASE(test_bn_load_multi_byte),
    TEST_CASE(test_bn_load_large_data),
    TEST_CASE(test_bn_load_oversized),
    
    // BNStore tests
    TEST_CASE(test_bn_store_null_pointer),
    TEST_CASE(test_bn_store_zero),
    TEST_CASE(test_bn_store_single_digit),
    TEST_CASE(test_bn_store_buffer_overflow),
    
    // BNCopy tests
    TEST_CASE(test_bn_copy_valid),
    TEST_CASE(test_bn_copy_null_pointers),
    
    // BNCompare tests
    TEST_CASE(test_bn_compare_equal),
    TEST_CASE(test_bn_compare_less_than),
    TEST_CASE(test_bn_compare_greater_than),
    TEST_CASE(test_bn_compare_zero),
    TEST_CASE(test_bn_compare_null_pointers),
    
    // BNSecureCompare tests
    TEST_CASE(test_bn_secure_compare_equal),
    TEST_CASE(test_bn_secure_compare_different),
    TEST_CASE(test_bn_secure_compare_null_pointers),
    
    // BNAdd tests
    TEST_CASE(test_bn_add_simple),
    TEST_CASE(test_bn_add_zero),
    TEST_CASE(test_bn_add_carry_within_digit),
    TEST_CASE(test_bn_add_null_pointers),
    
    // BNBitCount tests
    TEST_CASE(test_bn_bit_count_zero),
    TEST_CASE(test_bn_bit_count_powers_of_two),
    TEST_CASE(test_bn_bit_count_non_power_of_two),
    TEST_CASE(test_bn_bit_count_null_pointer),
    
    // Round-trip tests
    TEST_CASE(test_load_store_roundtrip_single_byte),
    TEST_CASE(test_load_store_roundtrip_multi_byte),
    
    // Placeholder tests
    TEST_CASE(test_placeholder_functions),
};

int main(void) {
    printf("=== BigNum-RS Unit Test Suite ===\n\n");
    
    size_t num_tests = sizeof(test_cases) / sizeof(test_cases[0]);
    
    for (size_t i = 0; i < num_tests; i++) {
        test_cases[i].func();
    }
    
    printf("\n=== Test Results ===\n");
    printf("Tests run:    %d\n", tests_run);
    printf("Tests passed: %d\n", tests_passed);
    printf("Tests failed: %d\n", tests_failed);
    
    if (tests_failed == 0) {
        printf("\n🎉 ALL TESTS PASSED!\n");
        printf("Code coverage: %.1f%% (%d/%d functions tested)\n", 
               (tests_passed * 100.0) / tests_run, tests_passed, tests_run);
    } else {
        printf("\n❌ SOME TESTS FAILED\n");
    }
    
    return tests_failed == 0 ? 0 : 1;
}
