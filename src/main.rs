#![feature(lang_items)]

#![deny(warnings)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

#![windows_subsystem="console"]

#![no_std]
#![no_main]

extern crate rlibc;

#[no_mangle]
pub extern "C" fn _aulldiv() -> ! { exit(10) }
#[no_mangle]
pub extern "C" fn _aullrem() -> ! { exit(11) }
#[no_mangle]
pub extern "C" fn strlen() -> ! { exit(12) }
#[no_mangle]
pub extern "C" fn _fltused() -> ! { exit(13) }
//#[no_mangle]
//pub extern fn _ZN4core10intrinsics17const_eval_select17hf03a2474bc3721cfE() { }

/*
#[lang="eh_personality"]
extern "C" fn eh_personality() {}

#[no_mangle]
pub extern "C" fn rust_eh_register_frames() {}

#[no_mangle]
pub extern "C" fn rust_eh_unregister_frames() {}
*/

//use arrayvec::ArrayVec;
use core::arch::asm;
use core::mem::{MaybeUninit, size_of, transmute};
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

fn dos_version() -> (u8, u8) {
    let mut major;
    let mut minor;
    unsafe {
        asm!(
            "int 0x21",
            in("ah") 0x30u8,
            lateout("ah") minor,
            lateout("al") major,
            lateout("cx") _,
            lateout("bx") _,
        );
    }
    (major, minor)
}

unsafe fn current_code_page() -> Result<u16, u16> {
    let mut res: u16;
    let mut cf: u8;
    let mut err: u16;
    asm!(
        "int 0x21",
        "mov {err:x}, ax",
        "lahf",
        err = lateout(reg) err,
        in("ah") 0x66u8,
        in("al") 0x01u8,
        lateout("ah") cf,
        lateout("bx") res,
        lateout("dx") _,
    );
    let ok = (cf & 0x01) == 0;
    if ok { Ok(res) } else { Err(err) }
}

/*
fn get_psp_address() -> u16 {
    let mut res: u16;
    unsafe {
        asm!(
            "int 0x21",
            in("ah") 0x62u8,
            lateout("bx") res,
        );
    }
    res
}

fn get_segment_base_address(selector: u16) -> Result<u32, u16> {
    let mut cf: u8;
    let mut err: u16;
    let mut hw: u16;
    let mut lw: u16;
    unsafe {
        asm!(
            "int 0x31",
            "mov {err:x}, ax",
            "lahf",
            err = lateout(reg) err,
            in("ax") 0x0006u16,
            in("bx") selector,
            lateout("ah") cf,
            lateout("al") _,
            lateout("cx") hw,
            lateout("dx") lw,
        );
    }
    let ok = (cf & 0x01) == 0;
    if ok { Ok(((hw as u32) << 16) | (lw as u32)) } else { Err(err) }
}
*/

#[inline]
fn p32<T>(p: *const T) -> u32 {
    assert!(size_of::<*const T>() == size_of::<u32>());
    let v = p as usize as u32;
    v
}

unsafe fn print(s: &[u8]) {
    asm!(
        "int 0x21",
        in("ah") 0x09u8,
        in("edx") p32(s.as_ptr()),
        lateout("al") _,
    );
}

unsafe fn open(filename: *const u8, mode: u8) -> Result<u16, u16> {
    let mut handle: u16;
    let mut cf: u8;
    asm!(
        "int 0x21",
        "mov {handle:x}, ax",
        "lahf",
        handle = lateout(reg) handle,
        in("ah") 0x3du8,
        in("al") mode,
        in("edx") p32(filename),
        lateout("ah") cf,
        lateout("al") _,
    );
    let ok = (cf & 0x01) == 0;
    if ok { Ok(handle) } else { Err(handle) }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "stdcall" fn mainCRTStartup() -> ! {
    let dos = dos_version();
    if dos.0 < 3 || dos.0 == 3 && dos.1 < 30 {
        unsafe { print(b"DOS < 3.3$"); }
        exit(33);
    }
    let code_page = (unsafe { current_code_page() })
        .unwrap_or_else(|_| { unsafe { print(b"Cannot determine code page.$") }; exit(1) });
    unsafe { print(b"Path$"); }
    let mut code_page_path: [MaybeUninit<u8>; 13] = unsafe { MaybeUninit::uninit().assume_init() };
    (&mut code_page_path[.. 9]).copy_from_slice(unsafe { transmute(&b"CODEPAGE\\"[..]) });
    code_page_path[9].write(b'0' + (code_page / 100) as u8);
    code_page_path[10].write(b'0' + ((code_page % 100) / 10) as u8);
    code_page_path[11].write(b'0' + (code_page % 10) as u8);
    code_page_path[12].write(0);
    unsafe { print(b"Opening$"); }
    /*
    let code_page_path: [u8; 13] = unsafe { transmute(code_page_path) };
    let _code_page = (unsafe { open(code_page_path.as_ptr(), 0x00) })
        .unwrap_or_else(|_| { unsafe { print(b"Cannot open code page file.$") }; exit(1) });
    unsafe { print(b"OK$"); }
    */
    exit(0)
}
