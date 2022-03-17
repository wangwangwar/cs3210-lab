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

fn is_power_of_two(num: usize) -> bool {
    num.next_power_of_two() == num
}