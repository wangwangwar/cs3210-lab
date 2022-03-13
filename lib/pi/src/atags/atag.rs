use core::slice;
use core::str;
use crate::atags::raw;

pub use crate::atags::raw::{Core, Mem};

/// An ATAG.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Atag {
    Core(raw::Core),
    Mem(raw::Mem),
    Cmd(&'static str),
    Unknown(u32),
    None,
}

impl Atag {
    /// Returns `Some` if this is a `Core` ATAG. Otherwise returns `None`.
    pub fn core(self) -> Option<Core> {
        match self {
            Self::Core(core) => Some(core),
            _ => None
        }
    }

    /// Returns `Some` if this is a `Mem` ATAG. Otherwise returns `None`.
    pub fn mem(self) -> Option<Mem> {
        match self {
            Self::Mem(mem) => Some(mem),
            _ => None
        }
    }

    /// Returns `Some` with the command line string if this is a `Cmd` ATAG.
    /// Otherwise returns `None`.
    pub fn cmd(self) -> Option<&'static str> {
        match self {
            Self::Cmd(cmd) => Some(cmd),
            _ => None
        }
    }

    fn get_str_len(str_pointer: *const u8) -> isize {
        let mut str_len: isize = 0;
        let mut str_pointer_idx = str_pointer;
        unsafe {
            loop {
                if *str_pointer_idx == '\0' as u8 {
                    break;
                }
                str_len += 1;
                str_pointer_idx = str_pointer_idx.add(1);
            }
        }
        str_len
    }
}

// FIXME: Implement `From<&raw::Atag> for `Atag`.
impl From<&'static raw::Atag> for Atag {
    fn from(atag: &'static raw::Atag) -> Atag {
        // FIXME: Complete the implementation below.
        unsafe {
            match (atag.tag, &atag.kind) {
                (raw::Atag::CORE, &raw::Kind { core }) => Self::Core(core),
                (raw::Atag::MEM, &raw::Kind { mem }) => Self::Mem(mem),
                (raw::Atag::CMDLINE, &raw::Kind { ref cmd }) => {
                    let cmd_str_pointer = &cmd.cmd as *const u8;
                    let cmd_str_len = Self::get_str_len(cmd_str_pointer);
                    let slice = slice::from_raw_parts(cmd_str_pointer, cmd_str_len as usize);
                    let str = str::from_utf8_unchecked(slice);
                    Atag::Cmd(str)
                },
                (raw::Atag::NONE, _) => Atag::None,
                (id, _) => Atag::Unknown(id),
            }
        }
    }
}
