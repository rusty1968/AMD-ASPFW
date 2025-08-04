use std::mem;

#[repr(C)]
struct KC_BIGNUM {
    Used: u32,
    Sign: u32, 
    Obfuscated: u32,
    Data: [u32; 80],
}

fn main() {
    println!("KC_BIGNUM size: {} bytes", mem::size_of::<KC_BIGNUM>());
    println!("KC_BIGNUM alignment: {} bytes", mem::align_of::<KC_BIGNUM>());
    println!("u32 alignment: {} bytes", mem::align_of::<u32>());
    
    // Test misaligned pointer
    let mut buffer = vec![0u8; mem::size_of::<KC_BIGNUM>() + 8];
    let base_ptr = buffer.as_mut_ptr();
    
    // Create aligned pointer
    let aligned_ptr = base_ptr as *mut KC_BIGNUM;
    println!("Aligned pointer: {:p} (alignment check: {})", 
             aligned_ptr, 
             (aligned_ptr as usize) % mem::align_of::<KC_BIGNUM>() == 0);
    
    // Create misaligned pointer (off by 1 byte)
    let misaligned_ptr = unsafe { base_ptr.add(1) } as *mut KC_BIGNUM;
    println!("Misaligned pointer: {:p} (alignment check: {})", 
             misaligned_ptr, 
             (misaligned_ptr as usize) % mem::align_of::<KC_BIGNUM>() == 0);
}
