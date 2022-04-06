use core::panicking::panic;

/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if !is_power_of_two(align) {
        panic("align is not a power of 2");
    }
    let next_multiple_of_align = addr.next_multiple_of(align);
    if next_multiple_of_align == addr { addr } else { next_multiple_of_align - align }
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2
/// or aligning up overflows the address.
pub fn align_up(addr: usize, align: usize) -> usize {
    if !is_power_of_two(align) {
        panic("align is not a power of 2");
    }
    addr.next_multiple_of(align)
}

pub fn is_power_of_two(num: usize) -> bool {
    num.next_power_of_two() == num
}

pub fn is_align(addr: usize, align: usize) -> bool {
    addr.next_multiple_of(align) == addr
}

/// A simple allocator that allocates based on size classes.
///   bin 0 (2^3 bytes)    : handles allocations in (0, 2^3]
///   bin 1 (2^4 bytes)    : handles allocations in (2^3, 2^4]
///   ...
///   bin 29 (2^32 bytes): handles allocations in (2^31, 2^32]
///   
///   map_to_bin(size) -> k
pub fn map_to_bin(size: usize) -> usize {
    return if size <= 8 {
        0usize
    } else {
        let log2 = size.log2();
        if 2usize.pow(log2) == size {
            (log2 - 3) as usize
        } else {
            (log2 - 2) as usize
        }
    }
}