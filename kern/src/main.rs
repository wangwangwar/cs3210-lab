#![feature(alloc_error_handler)]
//#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(auto_traits)]
#![feature(raw_vec_internals)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(negative_impls)]
#![feature(panic_info_message)]
#![feature(int_roundings)]
#![feature(core_panic)]

#[cfg(not(test))]
mod init;

extern crate alloc;

pub mod allocator;
pub mod console;
pub mod fs;
pub mod mutex;
pub mod shell;

use console::{Console, CONSOLE, kprintln};
use pi::uart::MiniUart;
use core::fmt::Write;
use pi::atags::Atags;

use allocator::Allocator;
use fs::FileSystem;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();

fn kmain() -> ! {
    for atag in Atags::get() {
        kprintln!("{:#?}", atag);
    }

    unsafe {
        ALLOCATOR.initialize();
        FILESYSTEM.initialize();
    }

    kprintln!("Welcome to cs3210!");
    shell::shell("> ");
}
