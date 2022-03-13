use core::panic::PanicInfo;
use crate::console::kprintln;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
   kprintln!("---------- PANIC ----------\n");
   let loc = _info.location().expect("location");
   kprintln!("FILE: {}\nLINE: {}\nCOL: {}", loc.file(), loc.line(), loc.column());
   let message = _info.message().expect("message");
   kprintln!("\n{}", message);
   loop {}
}
