#![feature(alloc_error_handler)]
//#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(auto_traits)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(negative_impls)]

#[cfg(not(test))]
mod init;

pub mod console;
pub mod mutex;
pub mod shell;

use console::{Console, CONSOLE, kprintln};
use pi::uart::MiniUart;
use core::fmt::Write;

// FIXME: You need to add dependencies here to
// test your drivers (Phase 2). Add them as needed.

fn kmain() -> ! {
    // FIXME: Start the shell.
    shell::shell("> ")
}
