#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include "include/bignum_ffi.h"

int main() {
    printf("=== Diagnostic Tests for Bignum Issues ===\n\n");
    
    // Issue 1: BNStore multi-byte test
    printf("1. Testing BNStore multi-byte issue:\n");
    KC_BIGNUM bn;
    uint8_t input_data[] = {0xAB, 0xCD, 0xEF, 0x01};
    uint32_t status = BNLoad(&bn, input_data, sizeof(input_data));
    printf("   BNLoad status: %u\n", status);
    printf("   Loaded bignum: Used=%u, Data[0]=0x%08X, Data[1]=0x%08X\n", 
           bn.Used, bn.Data[0], bn.Data[1]);
    
    uint8_t output[8];
    memset(output, 0, sizeof(output));
    status = BNStore(&bn, output, sizeof(output));
    printf("   BNStore status: %u\n", status);
    printf("   Expected: ");
    for (int i = 0; i < 4; i++) printf("0x%02X ", input_data[i]);
    printf("\n   Output:   ");
    for (int i = 0; i < 8; i++) printf("0x%02X ", output[i]);
    printf("\n");
    
    // Issue 2: Round-trip with 0xFF pattern
    printf("\n2. Testing round-trip with 0xFF pattern:\n");
    uint8_t ff_data[] = {0xFF, 0xFF, 0xFF, 0xFF};
    status = BNLoad(&bn, ff_data, sizeof(ff_data));
    printf("   BNLoad status: %u\n", status);
    printf("   Loaded bignum: Used=%u, Data[0]=0x%08X, Data[1]=0x%08X\n", 
           bn.Used, bn.Data[0], bn.Data[1]);
    
    memset(output, 0, sizeof(output));
    status = BNStore(&bn, output, sizeof(output));
    printf("   BNStore status: %u\n", status);
    printf("   Expected: ");
    for (int i = 0; i < 4; i++) printf("0x%02X ", ff_data[i]);
    printf("\n   Output:   ");
    for (int i = 0; i < 8; i++) printf("0x%02X ", output[i]);
    printf("\n");
    
    // Issue 3: BNAdd carry test
    printf("\n3. Testing BNAdd carry issue:\n");
    KC_BIGNUM a, b, result;
    BN_SET_INT(a, 0x0FFFFFFF); // Max 28-bit value
    BN_SET_INT(b, 1);
    printf("   a: Used=%u, Data[0]=0x%08X\n", a.Used, a.Data[0]);
    printf("   b: Used=%u, Data[0]=0x%08X\n", b.Used, b.Data[0]);
    
    status = BNAdd(&result, &a, &b);
    printf("   BNAdd status: %u\n", status);
    printf("   Result: Used=%u, Data[0]=0x%08X, Data[1]=0x%08X\n", 
           result.Used, result.Data[0], result.Data[1]);
    printf("   Expected: Used=2, Data[0]=0x00000000, Data[1]=0x00000001\n");
    
    return 0;
}
