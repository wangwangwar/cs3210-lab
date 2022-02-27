use core::str;
use stack_vec::StackVec;

use crate::console::{kprint, kprintln, CONSOLE};

// Accept commands at most 512 bytes in length 
const MAX_BYTES_PER_COMMAND: usize = 512;

// Accept at most 64 arguments per command
const MAX_ARGS_PER_COMMAND: usize = 64;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
pub fn shell(prefix: &str) -> ! {
    loop {
        kprint!("{}", prefix);
        let mut buf = [0u8; MAX_BYTES_PER_COMMAND];
        let cmd_str = get_cmd_str(&mut buf);
        let str_buf: &mut [&str] = &mut [""; MAX_ARGS_PER_COMMAND];
        kprintln!("");
        match Command::parse(cmd_str, str_buf) {
            Ok(cmd) if cmd.args[0] == "echo" => echo(&cmd.args[1..]).expect("echo error"),
            Ok(cmd) => kprintln!("unknown command: {}", cmd.args[0]),
            Err(_) => kprintln!("error"),
        }
    }
}

fn get_cmd_str<'a> (buf: &'a mut [u8; MAX_BYTES]) -> &'a str {
    let mut i = 0;
    loop {
        match CONSOLE.lock().read_byte() {
            b'\r' | b'\n' => break,
            8 | 127 => {
                // backspace
                if i <= 0 {
                    continue;
                }
                kprint!("{} {}", 8 as char, 8 as char);
                i -= 1;
            },
            _ if i >= MAX_BYTES_PER_COMMAND => continue,
            byte => {
                buf[i] = byte;
                kprint!("{}", byte as char);
                i += 1;
            },
        }
    }
    let str = str::from_utf8(&buf[..i]).unwrap();
    str.clone()
}

fn echo(args: &[&str]) -> Result<(), Error> {
    for arg in args {
        kprint!("{} ", arg);
    }
    kprintln!("");
    Ok(())
}