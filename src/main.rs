#![feature(core_ffi_c)]

#![deny(warnings)]

#![windows_subsystem="console"]

#![no_std]
#![no_main]

use core::arch::asm;
use core::ffi::c_int;
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

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn mainCRTStartup() -> c_int {
    0
}
