mod align_util {
    use crate::allocator::util::{align_down, align_up};

    #[test]
    fn test_align_down() {
        assert_eq!(align_down(0, 2), 0);
        assert_eq!(align_down(0, 8), 0);
        assert_eq!(align_down(0, 1 << 5), 0);

        assert_eq!(align_down(1 << 10, 1 << 10), 1 << 10);
        assert_eq!(align_down(1 << 20, 1 << 10), 1 << 20);
        assert_eq!(align_down(1 << 23, 1 << 4), 1 << 23);

        assert_eq!(align_down(1, 1 << 4), 0);
        assert_eq!(align_down(10, 1 << 4), 0);

        assert_eq!(align_down(0xFFFF, 1 << 2), 0xFFFC);
        assert_eq!(align_down(0xFFFF, 1 << 3), 0xFFF8);
        assert_eq!(align_down(0xFFFF, 1 << 4), 0xFFF0);
        assert_eq!(align_down(0xFFFF, 1 << 5), 0xFFE0);
        assert_eq!(align_down(0xAFFFF, 1 << 8), 0xAFF00);
        assert_eq!(align_down(0xAFFFF, 1 << 12), 0xAF000);
        assert_eq!(align_down(0xAFFFF, 1 << 16), 0xA0000);
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 2), 0);
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(0, 1 << 5), 0);

        assert_eq!(align_up(1 << 10, 1 << 10), 1 << 10);
        assert_eq!(align_up(1 << 20, 1 << 10), 1 << 20);
        assert_eq!(align_up(1 << 23, 1 << 4), 1 << 23);

        assert_eq!(align_up(1, 1 << 4), 1 << 4);
        assert_eq!(align_up(10, 1 << 4), 1 << 4);

        assert_eq!(align_up(0xFFFF, 1 << 2), 0x10000);
        assert_eq!(align_up(0xFFFF, 1 << 3), 0x10000);
        assert_eq!(align_up(0xFFFF, 1 << 4), 0x10000);
        assert_eq!(align_up(0xAFFFF, 1 << 12), 0xB0000);

        assert_eq!(align_up(0xABCDAB, 1 << 2), 0xABCDAC);
        assert_eq!(align_up(0xABCDAB, 1 << 4), 0xABCDB0);
        assert_eq!(align_up(0xABCDAB, 1 << 8), 0xABCE00);
        assert_eq!(align_up(0xABCDAB, 1 << 12), 0xABD000);
        assert_eq!(align_up(0xABCDAB, 1 << 16), 0xAC0000);
    }

    #[test]
    #[should_panic]
    fn test_panics_1() {
        align_down(0xFFFF0000, 7);
    }
    #[test]
    #[should_panic]
    fn test_panics_2() {
        align_down(0xFFFF0000, 123);
    }
    #[test]
    #[should_panic]
    fn test_panics_3() {
        align_up(0xFFFF0000, 7);
    }
    #[test]
    #[should_panic]
    fn test_panics_4() {
        align_up(0xFFFF0000, 456);
    }
}

mod allocator {
    use crate::allocator::{bin, bump, LocalAlloc};

    use std::alloc::{self, Layout};
    use std::alloc::handle_alloc_error;
    use std::marker::PhantomData;
    use std::mem;
    use std::ops::{Deref, DerefMut};
    use std::ptr::{self, NonNull, Unique};
    use std::collections::TryReserveError;
    use std::collections::TryReserveErrorKind::*;

    pub struct RawVec<T> {
        ptr: NonNull<T>,
        cap: usize,
        _marker: PhantomData<T>,
    }

    unsafe impl<T: Send> Send for RawVec<T> {}
    unsafe impl<T: Sync> Sync for RawVec<T> {}

    impl<T> RawVec<T> {
        fn new() -> Self {
            assert!(mem::size_of::<T>() != 0, "TODO: implement ZST support");
            RawVec {
                ptr: NonNull::dangling(),
                cap: 0,
                _marker: PhantomData,
            }
        }

        fn with_capacity(capacity: usize) -> Self {
            if mem::size_of::<T>() == 0 {
                Self {
                    ptr: NonNull::dangling(),
                    cap: 0,
                    _marker: PhantomData,
                }
            } else {
                // We avoid `unwrap_or_else` here because it bloats the amount of
                // LLVM IR generated.
                let layout = match Layout::array::<T>(capacity) {
                    Ok(layout) => layout,
                    Err(_) => capacity_overflow(),
                };
                match alloc_guard(layout.size()) {
                    Ok(_) => {}
                    Err(_) => capacity_overflow(),
                }
                let ptr = unsafe { alloc::alloc(layout) };
                let ptr2 = match NonNull::new(ptr as *mut T) {
                    Some(ptr) => ptr,
                    None => handle_alloc_error(layout),
                };

                // Allocators currently return a `NonNull<[u8]>` whose length
                // matches the size requested. If that ever changes, the capacity
                // here should change to `ptr.len() / mem::size_of::<T>()`.
                Self {
                    ptr: ptr2,
                    cap: capacity,
                    _marker: PhantomData
                }
            }
        }

        #[inline]
        pub fn ptr(&self) -> *mut T {
            self.ptr.as_ptr()
        }
    }

    impl<T> Drop for RawVec<T> {
        fn drop(&mut self) {
            if self.cap != 0 {
                let layout = Layout::array::<T>(self.cap).unwrap();
                unsafe {
                    alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
                }
            }
        }
    }

    #[inline]
    fn alloc_guard(alloc_size: usize) -> Result<(), TryReserveError> {
        if usize::BITS < 64 && alloc_size > isize::MAX as usize {
            Err(CapacityOverflow.into())
        } else {
            Ok(())
        }
    }

    #[cfg(not(no_global_oom_handling))]
    fn capacity_overflow() -> ! {
        panic!("capacity overflow");
    }

    macro_rules! test_allocators {
        (@$kind:ident, $name:ident, $mem:expr, |$info:pat| $block:expr) => {
            #[test]
            fn $name() {
                let mem: RawVec<u8> = RawVec::with_capacity($mem);
                let start = mem.ptr() as usize;
                let end = start + $mem;

                let allocator = $kind::Allocator::new(start, end);
                let $info = (start, end, allocator);

                #[allow(unused_unsafe)]
                unsafe {
                    $block
                }
            }
        };

        ($bin:ident, $bump:ident, $mem:expr, |$info:pat| $block:expr) => (
            test_allocators!(@bin, $bin, $mem, |$info| $block);
            test_allocators!(@bump, $bump, $mem, |$info| $block);
        );
    }

    macro layout($size:expr, $align:expr) {
        Layout::from_size_align($size, $align).unwrap()
    }

    macro test_layouts($layouts:expr, $start:expr, $end:expr, $a:expr) {
        let (layouts, start, end, mut a) = ($layouts, $start, $end, $a);

        let mut pointers: Vec<(usize, Layout)> = vec![];
        for layout in &layouts {
            let ptr = a.alloc(layout.clone());
            assert!(!ptr.is_null());
            pointers.push((ptr as usize, layout.clone()));
        }

        // Check that we have allocations after 'start' and before 'end'.
        for &(ptr, ref layout) in &pointers {
            assert!(
                ptr >= start,
                "allocated {:x} after start ({:x})",
                ptr,
                start
            );
            assert!(
                ptr + layout.size() <= end,
                "{:x} + {:x} exceeds the bounds of {:x}",
                ptr,
                layout.size(),
                end
            );
        }

        // Check that we have non-overlapping allocations.
        pointers.sort_by_key(|&(ptr, _)| ptr);
        for window in pointers.windows(2) {
            let (&(ptr_a, ref layout_a), &(ptr_b, _)) = (&window[0], &window[1]);
            assert!(
                ptr_b - ptr_a >= layout_a.size(),
                "memory region {:x} - {:x} does not fit {}",
                ptr_a,
                ptr_b,
                layout_a.size()
            );
        }

        // Check alignment.
        for &(ptr, ref layout) in &pointers {
            assert!(
                ptr % layout.align() == 0,
                "{:x} is not aligned to {}",
                ptr,
                layout.align()
            );
        }
    }

    test_allocators!(bin_exhausted, bump_exhausted, 128, |(_, _, mut a)| {
        let result = a.alloc(layout!(1024, 128));
        assert!(result.is_null());
    });

    test_allocators!(bin_alloc, bump_alloc, 8 * (1 << 20), |(start, end, a)| {
        let layouts = [
            layout!(16, 16),
            layout!(16, 128),
            layout!(16, 256),
            layout!(4, 256),
            layout!(1024, 16),
            layout!(1024, 4),
            layout!(1024, 128),
            layout!(2048, 8),
            layout!(2049, 8),
            layout!(2050, 8),
            layout!(4095, 4),
            layout!(4096, 4),
            layout!(4096, 4),
            layout!(4096, 4096),
            layout!(16, 4096),
            layout!(8192, 4096),
            layout!(8192, 8),
            layout!(8192, 8),
        ];

        // Test a few specially chosen layouts.
        test_layouts!(layouts, start, end, a);
    });

    test_allocators!(bin_alloc_2, bump_alloc_2, 16 * (1 << 20), |(
        start,
        end,
        a,
    )| {
        let mut layouts = vec![];
        for i in 1..1024 {
            layouts.push(layout!(i * 8, 16));
        }

        // Ensure ~contiguous allocations are properly handled.
        test_layouts!(layouts, start, end, a);
    });

    fn scribble(ptr: *mut u8, size: usize) {
        unsafe {
            ::core::ptr::write_bytes(ptr, 0xAF, size);
        }
    }

    test_allocators!(bin_dealloc_s, bump_dealloc_s, 4096, |(_, _, mut a)| {
        let layouts = [layout!(16, 16), layout!(16, 128), layout!(16, 256)];

        let mut pointers: Vec<(usize, Layout)> = vec![];
        for layout in &layouts {
            let ptr = a.alloc(layout.clone());
            assert!(!ptr.is_null());
            scribble(ptr, layout.size());
            pointers.push((ptr as usize, layout.clone()));
        }

        // Just check that deallocation doesn't panic.
        for (ptr, layout) in pointers {
            scribble(ptr as *mut u8, layout.size());
            a.dealloc(ptr as *mut u8, layout);
        }
    });

    test_allocators!(@bin, bin_dealloc_1, 65536, |(_, _, mut a)| {
        let layouts = [
            layout!(16, 16),
            layout!(16, 256),
            layout!(32, 4),
            layout!(32, 1024),
            layout!(4, 1024),
            layout!(4, 32),
        ];

        // tests for resonable internal fragmentation, reuse of aligned blocks,
        // and proper alignment after binning
        for (i, layout) in layouts.iter().enumerate() {
            let mut ptrs = vec![];
            for _ in 0..(25 + i * 2) {
                let ptr = a.alloc(layout.clone());
                assert!(!ptr.is_null());
                assert!(ptr as usize % layout.align() == 0,
                    "{:x} is not aligned to {}", ptr as usize, layout.align());
                scribble(ptr, layout.size());
                ptrs.push((ptr, layout.clone()));
            }

            for (ptr, layout) in ptrs {
                a.dealloc(ptr, layout);
            }
        }

        for _ in 0..500 {
            for layout in &layouts {
                let ptr = a.alloc(layout.clone());
                assert!(!ptr.is_null());
                scribble(ptr, layout.size());
                assert!(ptr as usize % layout.align() == 0,
                    "{:x} is not aligned to {}", ptr as usize, layout.align());
                a.dealloc(ptr, layout.clone());
            }
        }
    });

    test_allocators!(@bin, bin_dealloc_2, 8192, |(_, _, mut a)| {
        let layouts = [
            layout!(3072, 16),
            layout!(512, 32),
        ];

        // ensure we can reuse freed memory. also tests that the allocator has
        // resonable internal fragmentation
        for _ in 0..1000 {
            let mut ptrs = vec![];
            for layout in &layouts {
                let ptr = a.alloc(layout.clone());
                assert!(!ptr.is_null());
                scribble(ptr, layout.size());
                ptrs.push(ptr as usize);
            }

            for (layout, ptr) in layouts.iter().zip(ptrs.into_iter()) {
                scribble(ptr as *mut u8, layout.size());
                a.dealloc(ptr as *mut u8, layout.clone());
            }
        }
    });
}

mod linked_list {
    use crate::allocator::linked_list::LinkedList;

    #[test]
    fn example_1() {
        let address_1 = (&mut (1 as usize)) as *mut usize;
        let address_2 = (&mut (2 as usize)) as *mut usize;

        let mut list = LinkedList::new();
        unsafe {
            list.push(address_1);
            list.push(address_2);
        }

        assert_eq!(list.peek(), Some(address_2));
        assert_eq!(list.pop(), Some(address_2));
        assert_eq!(list.pop(), Some(address_1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn example_2() {
        let address_1 = (&mut (1 as usize)) as *mut usize;
        let address_2 = (&mut (2 as usize)) as *mut usize;
        let address_3 = (&mut (3 as usize)) as *mut usize;

        let mut list = LinkedList::new();
        unsafe {
            list.push(address_1);
            list.push(address_2);
            list.push(address_3);
        }

        for node in list.iter_mut() {
            if node.value() == address_2 {
                node.pop();
            }
        }

        assert_eq!(list.pop(), Some(address_3));
        assert_eq!(list.pop(), Some(address_1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn example_3() {
        let address_1 = (&mut (1 as usize)) as *mut usize;
        let address_2 = (&mut (2 as usize)) as *mut usize;
        let address_3 = (&mut (3 as usize)) as *mut usize;

        let mut list = LinkedList::new();
        unsafe {
            list.push(address_1);
            list.push(address_2);
            list.push(address_3);
        }

        for node in list.iter_mut() {
            if node.value() == address_2 {
                node.pop();
            }
        }

        {
            let mut iter = list.iter();
            assert_eq!(iter.next(), Some(address_3));
            assert_eq!(iter.next(), Some(address_1));
            assert_eq!(iter.next(), None);
        }

        for node in list.iter_mut() {
            if node.value() == address_1 {
                node.pop();
            }
        }

        {
            let mut iter = list.iter();
            assert_eq!(iter.next(), Some(address_3));
            assert_eq!(iter.next(), None);
        }

        for node in list.iter_mut() {
            if node.value() == address_3 {
                node.pop();
            }
        }

        let mut iter = list.iter();
        assert_eq!(iter.next(), None);
    }
}
