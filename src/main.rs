#![feature(start)]

#![deny(warnings)]

#![windows_subsystem="console"]

#![no_std]

use core::arch::asm;
use core::panic::PanicInfo;

fn exit(return_code: u8) -> ! {
    unsafe {
        asm!(
            "mov ah, 0x4C",
            "int 0x21",
            in("al") return_code,
            out("ah") _,
        );
    }
    loop { }
}

#[panic_handler]
pub extern fn panic(_info: &PanicInfo) -> ! {
    exit(99)
}

#[no_mangle]
pub extern "C" fn rust_eh_register_frames() { }

#[no_mangle]
pub extern "C" fn rust_eh_unregister_frames() { }

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    0
}
