use core::alloc::Layout;
use core::fmt;
use core::ptr;

use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;
use crate::console::kprint;

/// A simple allocator that allocates based on size classes.
///   bin 0 (2^3 bytes)    : handles allocations in (0, 2^3]
///   bin 1 (2^4 bytes)    : handles allocations in (2^3, 2^4]
///   ...
///   bin 29 (2^32 bytes): handles allocations in (2^31, 2^32]
///   
///   map_to_bin(size) -> k
///   

pub struct Allocator {
    current: usize,
    start: usize,
    end: usize,
    bins: [LinkedList; 30],
    allocations: usize,
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new<'a>(start: usize, end: usize) -> Allocator {
        Allocator {
            current: start,
            start,
            end,
            bins: [LinkedList::new(); 30],
            allocations: 0,
        }
    }
}

impl LocalAlloc for Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning null pointer (`core::ptr::null_mut`)
    /// indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's
    /// size or alignment constraints.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        kprint!("alloc layout {}, {}\n", layout.size(), layout.align());
        if !is_power_of_two(layout.align()) {
            return ptr::null_mut();
        }
        let bin_index = map_to_bin(layout.size());
        kprint!("alloc bin_index: {}\n", bin_index);
        if bin_index > 29 {
            return ptr::null_mut();
        }
        let bin_list = &mut self.bins.as_mut()[bin_index];
        kprint!("alloc bin_list: {:?}\n", bin_list);
        for item in bin_list.iter_mut() {
            let ptr = align_up(item.value() as usize, layout.align());
            if is_align(item.value() as usize, layout.align()) {
                kprint!("alloc find item: {:?}\n", item.value());
                self.allocations += 1;
                return item.pop() as *mut u8;
            }
        }
        kprint!("alloc bin_list: {:?}\n", bin_list);

        let ptr = align_up(self.current, layout.align());
        let size = 2usize.pow((bin_index + 3) as u32);
        kprint!("alloc ptr: {:p}, size: {}\n", ptr as *mut u8, size);
        kprint!("alloc ptr + size: {:p}, current: {:p}, end: {:p}\n", (ptr + size) as *mut u8, self.current as *mut u8, self.end as *mut u8);
        if ptr + size > self.end {
            kprint!("alloc ptr + size > end\n");
            return ptr::null_mut();
        }
        self.current = ptr.saturating_add(size);
        self.allocations += 1;
        return ptr as *mut u8;
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        kprint!("dealloc, ptr: {:p}, layout: {} {}\n", ptr, layout.size(), layout.align());
        let bin_list = &mut self.bins.as_mut()[map_to_bin(layout.size())];
        bin_list.push(ptr as *mut usize);
        kprint!("dealloc bin_list: {:?}\n", bin_list);
        self.allocations -= 1;
        kprint!("dealloc allocations: {:?}\n", self.allocations);
        if self.allocations == 0 {
            self.current = self.start;
            for bin in &mut self.bins {
                kprint!("dealloc bin: {:?}\n", bin);
                while bin.pop().is_some() {
                    kprint!("dealloc pop");
                }
            }
        }
    }
}

// FIXME: Implement `Debug` for `Allocator`.
