#![feature(naked_functions)]

#![deny(warnings)]

#![windows_subsystem="console"]

#![no_std]
#![no_main]

use arrayvec::ArrayVec;
use core::arch::asm;
use core::mem::size_of;
use core::panic::PanicInfo;
//use null_terminated::Nul;

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

/*
fn open(filename: &Nul<u8>, mode: u8) -> Result<u16, u16> {
    let mut handle: u16;
    let mut cf: u8;
    unsafe {
        asm!(
            "int 0x21",
            "mov {handle:x}, ax",
            "lahf",
            handle = lateout(reg) handle,
            in("ah") 0x3du8,
            in("al") mode,
            in("edx") p32(filename.as_ptr()),
            lateout("ah") cf,
            lateout("al") _,
        );
    }
    let ok = (cf & 0x01) == 0;
    if ok { Ok(handle) } else { Err(handle) }
}
*/

struct CmdReader(*const u8);

impl Iterator for CmdReader {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let b = unsafe { *self.0 };
        if b == 0x0D { return None; }
        self.0 = unsafe { self.0.offset(1) };
        Some(b)
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn mainCRTStartup() -> ! {
    let dos = dos_version();
    if dos.0 < 3 || dos.0 == 3 && dos.1 < 30 {
        unsafe { print(b"DOS < 3.3$"); }
        exit(33);
    }
    let _code_page = (unsafe { current_code_page() })
        .unwrap_or_else(|_| { unsafe { print(b"Cannot determine code page.$") }; exit(1) });
    let psp = get_psp_address();
    let psp = get_segment_base_address(psp)
        .unwrap_or_else(|_| { unsafe { print(b"Cannot get PSP address.$"); exit(1); } });
    let cmd = CmdReader((psp + 0x81) as *const u8);
    let mut cmd = cmd.collect::<ArrayVec<_, 128>>();
    cmd.push(b'$');
    unsafe { print(&cmd[..]); }
    unsafe { print(b"OK$"); }
    exit(0)
}
