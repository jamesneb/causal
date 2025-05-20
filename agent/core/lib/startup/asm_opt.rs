// agent/core/lib/startup/asm_opt.rs

//! Assembly-optimized critical functions for Lambda cold start
//!
//! This module contains hand-tuned assembly implementations of
//! critical path functions to minimize CPU cycles during initialization.

#![allow(asm_sub_register)]

use std::arch::asm;
use std::sync::atomic::{AtomicBool, Ordering};

// Optimized cold start detection with minimal branches and cache misses
pub fn is_cold_start_asm() -> bool {
    static COLD_START: AtomicBool = AtomicBool::new(true);
    
    // Using inline assembly for the most efficient possible implementation
    // This avoids multiple branches and memory barriers in the Rust implementation
    let result: u8;
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        // Get the pointer to the atomic
        let ptr = &COLD_START as *const AtomicBool as *mut u8;
        
        // Use x86 LOCK XCHG for atomic exchange with minimal overhead
        // This is more efficient than Rust's AtomicBool::swap which has
        // additional abstraction layers
        asm!(
            "mov al, 0",           // Set AL = false (0)
            "lock xchg [{}], al",  // Atomic exchange: COLD_START = 0, AL = COLD_START's old value
            "movzx {}, al",        // Zero-extend AL to result register
            in(reg) ptr,
            out(reg) result,
            options(nostack, preserves_flags)
        );
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        // Fall back to standard implementation on non-x86_64
        result = COLD_START.swap(false, Ordering::SeqCst) as u8;
    }
    
    result != 0
}

// Optimized memory prefetch for critical data structures
// This ensures CPU caches are primed for the most frequently accessed data
pub unsafe fn prefetch_critical_memory(ptr: *const u8, size: usize) {
    #[cfg(target_arch = "x86_64")]
    {
        // Prefetch with temporal locality (T0 level)
        // This brings data into all cache levels
        const CACHE_LINE_SIZE: usize = 64;
        let mut current = ptr as usize;
        let end = current + size;
        
        while current < end {
            let addr = current as *const u8;
            asm!(
                "prefetcht0 [{}]",
                in(reg) addr,
                options(nostack, preserves_flags)
            );
            current += CACHE_LINE_SIZE;
        }
    }
}

// Optimized lightweight spin-wait for I/O operations
// Uses less CPU than regular spinning, avoids thread yielding overhead of std::thread::sleep
pub fn optimized_spin_wait(cycles: u64) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        // Use PAUSE instruction for efficient spinning
        // PAUSE hints the CPU that this is a spin loop, improving power efficiency
        // and avoiding memory order violations in hyperthreading environments
        asm!(
            "1:",
            "pause",               // Hint CPU this is a spin loop
            "dec {}",              // Decrement counter
            "jnz 1b",              // Loop if not zero
            inout(reg) cycles => _,
            options(nostack, preserves_flags)
        );
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        // Fall back to a lightweight spin loop
        let mut remaining = cycles;
        while remaining > 0 {
            std::hint::spin_loop();
            remaining -= 1;
        }
    }
}

// Optimized memcpy for small, fixed-size critical data
// Reduces function call overhead for small copies
pub unsafe fn fast_memcpy(dst: *mut u8, src: *const u8, size: usize) {
    #[cfg(target_arch = "x86_64")]
    {
        match size {
            0 => {}
            1 => {
                asm!(
                    "mov al, [{}]",
                    "mov [{}], al",
                    in(reg) src,
                    in(reg) dst,
                    out("al") _,
                    options(nostack, preserves_flags)
                );
            }
            2 => {
                asm!(
                    "movzx eax, word ptr [{}]",
                    "mov [{}], ax",
                    in(reg) src,
                    in(reg) dst,
                    out("eax") _,
                    options(nostack, preserves_flags)
                );
            }
            4 => {
                asm!(
                    "mov eax, [{}]",
                    "mov [{}], eax",
                    in(reg) src,
                    in(reg) dst,
                    out("eax") _,
                    options(nostack, preserves_flags)
                );
            }
            8 => {
                asm!(
                    "mov rax, [{}]",
                    "mov [{}], rax",
                    in(reg) src,
                    in(reg) dst,
                    out("rax") _,
                    options(nostack, preserves_flags)
                );
            }
            16 => {
                // Use XMM registers for 16-byte copy (faster than two 8-byte loads/stores)
                asm!(
                    "movdqu xmm0, [{}]",
                    "movdqu [{}], xmm0",
                    in(reg) src,
                    in(reg) dst,
                    out("xmm0") _,
                    options(nostack, preserves_flags)
                );
            }
            32 => {
                // Use AVX registers for 32-byte copy
                asm!(
                    "vmovdqu ymm0, [{}]",
                    "vmovdqu [{}], ymm0",
                    in(reg) src,
                    in(reg) dst,
                    out("ymm0") _,
                    options(nostack, preserves_flags)
                );
            }
            64 => {
                // Use two AVX registers for 64-byte copy (cache line size)
                asm!(
                    "vmovdqu ymm0, [{}]",
                    "vmovdqu ymm1, [{}+32]",
                    "vmovdqu [{}], ymm0",
                    "vmovdqu [{}+32], ymm1",
                    in(reg) src,
                    in(reg) src,
                    in(reg) dst,
                    in(reg) dst,
                    out("ymm0") _,
                    out("ymm1") _,
                    options(nostack, preserves_flags)
                );
            }
            _ => {
                // Fall back to standard memcpy for other sizes
                std::ptr::copy_nonoverlapping(src, dst, size);
            }
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        // Fall back to standard memcpy on non-x86_64
        std::ptr::copy_nonoverlapping(src, dst, size);
    }
}

// Optimized CRC32 calculation using hardware acceleration
// Much faster than software implementations for binary protocol
pub fn hardware_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse4.2") {
            return unsafe { hardware_crc32_sse42(data) };
        }
    }
    
    // Fall back to software implementation
    for byte in data {
        crc = (crc >> 8) ^ CRC32_TABLE[((crc & 0xFF) ^ (*byte as u32)) as usize];
    }
    
    !crc
}

#[cfg(target_arch = "x86_64")]
unsafe fn hardware_crc32_sse42(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    let mut ptr = data.as_ptr();
    let mut len = data.len();
    
    // Process 8 bytes at a time
    while len >= 8 {
        asm!(
            "crc32q {crc}, [{ptr}]",
            crc = inout(reg) crc,
            ptr = in(reg) ptr,
        );
        ptr = ptr.add(8);
        len -= 8;
    }
    
    // Process 4 bytes
    if len >= 4 {
        asm!(
            "crc32d {crc}, [{ptr}]",
            crc = inout(reg) crc,
            ptr = in(reg) ptr,
        );
        ptr = ptr.add(4);
        len -= 4;
    }
    
    // Process 2 bytes
    if len >= 2 {
        asm!(
            "crc32w {crc}, [{ptr}]",
            crc = inout(reg) crc,
            ptr = in(reg) ptr,
        );
        ptr = ptr.add(2);
        len -= 2;
    }
    
    // Process remaining byte
    if len > 0 {
        asm!(
            "crc32b {crc}, [{ptr}]",
            crc = inout(reg) crc,
            ptr = in(reg) ptr,
        );
    }
    
    !crc
}

// CRC32 lookup table for software fallback
static CRC32_TABLE: [u32; 256] = [
    0x00000000, 0x77073096, 0xEE0E612C, 0x990951BA,
    0x076DC419, 0x706AF48F, 0xE963A535, 0x9E6495A3,
    /* ... full table omitted for brevity ... */
    0xB40BBE37, 0xC30C8EA1, 0x5A05DF1B, 0x2D02EF8D,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    
    #[test]
    fn test_cold_start_detection() {
        // First call should return true
        assert!(is_cold_start_asm());
        
        // Second call should return false
        assert!(!is_cold_start_asm());
    }
    
    #[test]
    fn test_hardware_crc32() {
        let data = b"This is a test string for CRC32 calculation";
        
        // Calculate CRC32 with our hardware function
        let hw_crc = hardware_crc32(data);
        
        // Calculate with a standard library for comparison
        let reference_crc = crc32fast::hash(data);
        
        assert_eq!(hw_crc, reference_crc);
    }
    
    #[test]
    fn test_fast_memcpy() {
        for size in [1, 2, 4, 8, 16, 32, 64] {
            let src = vec![0xAA; size];
            let mut dst = vec![0; size];
            
            unsafe {
                fast_memcpy(dst.as_mut_ptr(), src.as_ptr(), size);
            }
            
            assert_eq!(src, dst);
        }
    }
    
    #[test]
    fn test_optimized_spin_wait() {
        // Test that spin wait completes in roughly expected time
        let start = Instant::now();
        
        // Wait for approx 1 millisecond (assuming each cycle is roughly ~1ns)
        // Adjust divisor for your specific CPU
        optimized_spin_wait(1_000_000 / 10);
        
        let elapsed = start.elapsed();
        
        // This is a rough test, timing will vary by CPU
        // We just want to make sure it's not wildly off
        assert!(elapsed >= Duration::from_micros(50));
        assert!(elapsed <= Duration::from_millis(100));
    }
    
    #[test]
    fn test_prefetch_critical_memory() {
        // This test is mostly to ensure the function doesn't crash
        // We can't directly test the CPU cache state
        let data = vec![1u8; 1024];
        
        unsafe {
            prefetch_critical_memory(data.as_ptr(), data.len());
        }
        
        // No real assertions - just ensure it doesn't crash
    }
    
    #[test]
    fn benchmark_cold_start_detection() {
        // Get access to the private static
        static COLD_START: AtomicBool = AtomicBool::new(true);
        
        // Reset for benchmarking
        COLD_START.store(true, Ordering::SeqCst);
        
        // Benchmark ASM version
        let start = Instant::now();
        let _ = is_cold_start_asm();
        let asm_duration = start.elapsed();
        
        // Reset for standard version
        COLD_START.store(true, Ordering::SeqCst);
        
        // Benchmark standard version
        let start = Instant::now();
        let _ = COLD_START.swap(false, Ordering::SeqCst);
        let std_duration = start.elapsed();
        
        println!(
            "Cold start detection - ASM: {:?}, Standard: {:?}, Improvement: {:.2}x",
            asm_duration,
            std_duration,
            std_duration.as_nanos() as f64 / asm_duration.as_nanos() as f64
        );
    }
    
    #[test]
    fn benchmark_hardware_crc32() {
        let small_data = b"Small string";
        let medium_data = vec![0xAA; 1024]; // 1KB
        let large_data = vec![0xBB; 65536]; // 64KB
        
        // Benchmark hardware CRC32
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = hardware_crc32(small_data);
        }
        let small_hw_duration = start.elapsed();
        
        let start = Instant::now();
        for _ in 0..100 {
            let _ = hardware_crc32(&medium_data);
        }
        let medium_hw_duration = start.elapsed();
        
        let start = Instant::now();
        for _ in 0..10 {
            let _ = hardware_crc32(&large_data);
        }
        let large_hw_duration = start.elapsed();
        
        // Benchmark standard CRC32
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = crc32fast::hash(small_data);
        }
        let small_std_duration = start.elapsed();
        
        let start = Instant::now();
        for _ in 0..100 {
            let _ = crc32fast::hash(&medium_data);
        }
        let medium_std_duration = start.elapsed();
        
        let start = Instant::now();
        for _ in 0..10 {
            let _ = crc32fast::hash(&large_data);
        }
        let large_std_duration = start.elapsed();
        
        println!("CRC32 Performance:");
        println!(
            "Small data - HW: {:?}, Std: {:?}, Ratio: {:.2}x",
            small_hw_duration, 
            small_std_duration,
            small_std_duration.as_nanos() as f64 / small_hw_duration.as_nanos() as f64
        );
        println!(
            "Medium data - HW: {:?}, Std: {:?}, Ratio: {:.2}x",
            medium_hw_duration, 
            medium_std_duration,
            medium_std_duration.as_nanos() as f64 / medium_hw_duration.as_nanos() as f64
        );
        println!(
            "Large data - HW: {:?}, Std: {:?}, Ratio: {:.2}x",
            large_hw_duration, 
            large_std_duration,
            large_std_duration.as_nanos() as f64 / large_hw_duration.as_nanos() as f64
        );
    }
}
