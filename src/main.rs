#![feature(lang_items)]
#![feature(panic_info_message)]

#![deny(warnings)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_parens)]
#![allow(unused_unsafe)]
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
use core::fmt::{self, Write};
use core::mem::{MaybeUninit, size_of, transmute};
use core::panic::PanicInfo;
use core::slice::{self};

mod dos {
    use core::arch::asm;
    use core::mem::{MaybeUninit, size_of};

    #[allow(non_snake_case)]
    #[inline]
    pub unsafe fn int_21h_ah_4Ch_exit(al_exit_code: u8) {
        asm!(
            "mov ah, 0x4C",
            "int 0x21",
            in("ax") al_exit_code as u16,
            out("ax") _,
        );
    }

    #[derive(Debug, Clone)]
    pub struct DosVer {
        pub ah_minor: u8,
        pub al_major: u8,
    }

    #[inline]
    pub fn int_21h_ah_30h_dos_ver() -> DosVer {
        let mut ax;
        unsafe {
            asm!(
                "int 0x21",
                in("ax") 0x3000u16,
                lateout("ax") ax,
                lateout("cx") _,
                lateout("bx") _,
            );
        }
        DosVer { ah_minor: (ax >> 8) as u8, al_major: ax as u8 }
    }

    #[derive(Debug, Clone)]
    pub struct CodePage {
        pub bx_active: u16,
        pub dx_default: u16,
    }

    #[derive(Debug, Clone)]
    pub struct AxErr {
        pub ax_err: u16,
    }

    const CF: u8 = 0x01;

    #[inline]
    pub unsafe fn int_21h_ax_6601h_code_page() -> Result<CodePage, AxErr> {
        let mut bx_active: u16;
        let mut dx_default: u16;
        let mut flags: u16;
        let mut ax_err: u16;
        asm!(
            "int 0x21",
            "mov {ax_err:x}, ax",
            "lahf",
            ax_err = lateout(reg) ax_err,
            in("ax") 0x6601u16,
            lateout("ax") flags,
            lateout("bx") bx_active,
            lateout("dx") dx_default,
        );
        if ((flags >> 8) as u8) & CF == 0 {
            Ok(CodePage { bx_active, dx_default })
        } else {
            Err(AxErr { ax_err })
        }
    }

    #[inline]
    fn p32<T>(p: *const T) -> u32 {
        assert!(size_of::<*const T>() == size_of::<u32>());
        let v = p as usize as u32;
        v
    }

    #[inline]
    pub unsafe fn int_21h_ah_09h_out_str(dx_str_24h: *const u8) {
        asm!(
            "int 0x21",
            in("ax") 0x0900u8,
            in("edx") p32(dx_str_24h),
            lateout("ax") _,
        );
    }

    #[derive(Debug, Clone)]
    pub struct AxHandle {
        pub ax_handle: u16,
    }

    #[allow(non_snake_case)]
    #[inline]
    pub unsafe fn int_21h_ah_3Dh_open(dx_path_z: *const u8, al_mode: u8) -> Result<AxHandle, AxErr> {
        let mut ax: u16;
        let mut flags: u16;
        asm!(
            "int 0x21",
            "mov {ax:x}, ax",
            "lahf",
            ax = lateout(reg) ax,
            in("ax") 0x3d00u16 | al_mode as u16,
            in("edx") p32(dx_path_z),
            lateout("ax") flags,
        );
        if ((flags >> 8) as u8) & CF == 0 {
            Ok(AxHandle { ax_handle: ax })
        } else {
            Err(AxErr { ax_err: ax })
        }
    }

    #[derive(Debug, Clone)]
    pub struct AxRead {
        pub ax_read: u16,
    }

    #[allow(non_snake_case)]
    #[inline]
    pub unsafe fn int_21h_ah_3Fh_read(bx_handle: u16, dx_cx_buf: &mut [MaybeUninit<u8>]) -> Result<AxRead, AxErr> {
        let mut flags: u16;
        let mut ax: u16;
        asm!(
            "int 0x21",
            "mov {ax:x}, ax",
            "lahf",
            ax = lateout(reg) ax,
            in("ax") 0x3F00u16
            in("bx") bx_handle,
            in("ecx") u16::try_from(dx_cx_buf.len()).unwrap() as u32,
            in("edx") p32(dx_cx_buf.as_mut_ptr()),
            lateout("ax") flags
        );
        if ((flags >> 8) as u8) & CF == 0 {
            Ok(AxRead { ax_read: ax })
        } else {
            Err(AxErr { ax_err: ax })
        }
    }

    #[derive(Debug, Clone)]
    pub struct AxSegment {
        pub ax_segment: u16,
    }

    #[derive(Debug, Clone)]
    pub struct AllocErr {
        pub ax_err: u16,
        pub bx_available_paragraphs: u16,
    }

    #[inline]
    pub unsafe fn int_21h_ah_48h_alloc(bx_paragraphs: u16) -> Result<AxSegment, AllocErr> {
        let mut ebx_paragraphs = bx_paragraphs as u32;
        let mut ax: u16;
        let mut flags: u16;
        asm!(
            "int 0x21",
            "mov {ax:x}, ax",
            "lahf",
            ax = lateout(reg) ax,
            in("ax") 0x4800u16,
            inlateout("ebx") ebx_paragraphs => ebx_paragraphs,
            lateout("ax") flags,
        );
        if ((flags >> 8) as u8) & CF == 0 {
            Ok(AxSegment { ax_segment: ax })
        } else {
            Err(AllocErr { ax_err: ax, bx_available_paragraphs: ebx_paragraphs as u16 })
        }
    }

    #[derive(Debug, Clone)]
    pub struct BxSegment {
        pub bx_segment: u16,
    }

    #[inline]
    pub unsafe fn int_21h_ah_62h_psp_addr() -> BxSegment {
        let mut bx_segment: u16;
        asm!(
            "int 0x21",
            in("ax") 0x6200u8,
            lateout("bx") bx_segment,
        );
        BxSegment { bx_segment }
    }

    #[derive(Debug, Clone)]
    pub struct CxDxAddr {
        pub cx_dx_addr: u32,
    }

    #[inline]
    pub unsafe fn int_31h_ax_0006h_segment_addr(bx_selector: u16) -> Result<CxDxAddr, AxErr> {
        let mut flags: u16;
        let mut ax_err: u16;
        let mut cx: u16;
        let mut dx: u16;
        asm!(
            "int 0x31",
            "mov {ax_err:x}, ax",
            "lahf",
            ax_err = lateout(reg) ax_err,
            in("ax") 0x0006u16,
            in("bx") bx_selector,
            lateout("ax") flags,
            lateout("cx") cx,
            lateout("dx") dx,
        );
        if ((flags >> 8) as u8) & CF == 0 {
            Ok(CxDxAddr { cx_dx_addr: ((cx as u32) << 16) | (dx as u32) })
        } else {
            Err(AxErr { ax_err })
        }
    }

    #[derive(Debug, Clone)]
    pub struct RmAlloc {
        pub ax_segment: u16,
        pub dx_selector: u16,
    }

    #[inline]
    pub unsafe fn int_31h_ax_0100h_rm_alloc(mut bx_paragraphs: u16) -> Result<RmAlloc, AllocErr> {
        let mut flags: u16;
        let mut ax: u16;
        let mut dx_selector: u16;
        asm!(
            "int 0x31",
            "mov {ax:x}, ax",
            "lahf",
            ax = lateout(reg) ax,
            in("ax") 0x0100u16,
            inlateout("bx") bx_paragraphs => bx_paragraphs,
            lateout("ax") flags,
            lateout("dx") dx_selector,
        );
        if ((flags >> 8) as u8) & CF == 0 {
            Ok(RmAlloc { ax_segment: ax, dx_selector })
        } else {
            Err(AllocErr { ax_err: ax, bx_available_paragraphs: bx_paragraphs })
        }
    }
}

use dos::*;

struct DosLastChanceWriter;

impl Write for DosLastChanceWriter {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let c = c as u32;
        let a = if c > 0x7F || c == '$' as u32 || c == '\r' as u32 {
            b'?'
        } else {
            c as u8
        };
        if a == b'\n' {
            unsafe { int_21h_ah_09h_out_str(b"\r\n$".as_ptr()); }
        } else {
            let buf = [a, b'$'];
            unsafe { int_21h_ah_09h_out_str(buf.as_ptr()); }
        }
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }
}

#[panic_handler]
pub extern fn panic(info: &PanicInfo) -> ! {
    let _ = DosLastChanceWriter.write_str("panic");
    if let Some(&message) = info.message() {
        let _ = DosLastChanceWriter.write_str(": ");
        let _ = DosLastChanceWriter.write_fmt(message);
    } else if let Some(message) = info.payload().downcast_ref::<&str>() {
        let _ = DosLastChanceWriter.write_str(": ");
        let _ = DosLastChanceWriter.write_str(message);
    } else {
        let _ = DosLastChanceWriter.write_str("!");
    }
    if let Some(location) = info.location() {
        let _ = writeln!(DosLastChanceWriter, " ({})", location);
    } else {
        let _ = writeln!(DosLastChanceWriter);
    }
    exit(99)
}

fn exit(exit_code: u8) -> ! {
    unsafe { int_21h_ah_4Ch_exit(exit_code); }
    loop { }
}

const CONVENTIONAL_MEMORY_REQUIRED: u16 = 6400;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "stdcall" fn mainCRTStartup() -> ! {
    let dos_ver = int_21h_ah_30h_dos_ver();
    if dos_ver.al_major < 3 || dos_ver.al_major == 3 && dos_ver.ah_minor < 30 {
        unsafe { int_21h_ah_09h_out_str(b"Error: DOS >= 3.3 required.\r\n$".as_ptr()); }
        exit(33);
    }
    let conventional_memory_size = match unsafe { int_31h_ax_0100h_rm_alloc(0xFFFF) }.err().unwrap() {
        AllocErr { ax_err: 8, bx_available_paragraphs } => bx_available_paragraphs,
        AllocErr { ax_err: 7, .. } => {
            unsafe { int_21h_ah_09h_out_str(b"Error: memory control block destroyed.\r\n$".as_ptr()); }
            exit(1)
        },
        AllocErr { ax_err: 0x8011, .. } => {
            unsafe { int_21h_ah_09h_out_str(b"Error: descriptor unavailable.\r\n$".as_ptr()); }
            exit(1)
        },
        _ => {
            unsafe { int_21h_ah_09h_out_str(b"Error: unknown memory error.\r\n$".as_ptr()); }
            exit(1)
        },
    };
    if conventional_memory_size < CONVENTIONAL_MEMORY_REQUIRED {
        unsafe { int_21h_ah_09h_out_str(b"Insuficient memory.\r\n$".as_ptr()); }
        exit(1);
    }
    let conventional_memory = (unsafe { int_31h_ax_0100h_rm_alloc(CONVENTIONAL_MEMORY_REQUIRED) }).unwrap_or_else(|_| {
        unsafe { int_21h_ah_09h_out_str(b"Cannot allocate memory.\r\n$".as_ptr()); }
        exit(1);
    });
    let a = unsafe { int_31h_ax_0006h_segment_addr(conventional_memory.dx_selector).unwrap().cx_dx_addr };
    assert!(a == (conventional_memory.ax_segment as u32) << 4);

    assert!(size_of::<usize>() == size_of::<u32>());
    let conventional_memory = unsafe { slice::from_raw_parts_mut(
        ((conventional_memory.ax_segment as u32) << 4) as *mut u8,
        ((CONVENTIONAL_MEMORY_REQUIRED as u32) << 4) as usize
    ) };
    let (code_page_memory, conventional_memory) = conventional_memory.split_at_mut(512);
    let code_page_n = (unsafe { int_21h_ax_6601h_code_page() }).unwrap_or_else(|_| {
        unsafe { int_21h_ah_09h_out_str(b"Cannot determine code page.\r\n$".as_ptr()); }
        exit(1);
    }).bx_active;
    let mut code_page: [MaybeUninit<u8>; 13] = unsafe { MaybeUninit::uninit().assume_init() };
    (&mut code_page[.. 9]).copy_from_slice(unsafe { transmute(&b"CODEPAGE\\"[..]) });
    code_page[9].write(b'0' + (code_page_n / 100) as u8);
    code_page[10].write(b'0' + ((code_page_n % 100) / 10) as u8);
    code_page[11].write(b'0' + (code_page_n % 10) as u8);
    code_page[12].write(0);
    let code_page: [u8; 13] = unsafe { transmute(code_page) };
    let code_page = (unsafe { int_21h_ah_3Dh_open(code_page.as_ptr(), 0x00) }).unwrap_or_else(|_| {
        unsafe { int_21h_ah_09h_out_str(b"Cannot open code page file.\r\n$".as_ptr()); }
        exit(1);
    }).ax_handle;
    let mut code_page_buf: &mut [MaybeUninit<u8>] = unsafe { transmute(&mut code_page_memory[..]) };
    unsafe { int_21h_ah_09h_out_str(b"OK\r\n$".as_ptr()); }
    loop {
        if code_page_buf.is_empty() {
            unsafe { int_21h_ah_09h_out_str(b"empty\r\n$".as_ptr()); }
            let mut byte: MaybeUninit<u8> = MaybeUninit::uninit();
            let read = (unsafe { int_21h_ah_3Fh_read(code_page, slice::from_mut(&mut byte)) }).unwrap_or_else(|_| {
                unsafe { int_21h_ah_09h_out_str(b"Cannot read code page file.\r\n$".as_ptr()); }
                exit(1);
            }).ax_read;
            if read != 0 {
                unsafe { int_21h_ah_09h_out_str(b"Invalid code page file: too big.\r\n$".as_ptr()); }
                exit(1);
            }
            break;
        }
        unsafe { int_21h_ah_09h_out_str(b"go\r\n$".as_ptr()); }
        let read = (unsafe { int_21h_ah_3Fh_read(code_page, code_page_buf) }).unwrap_or_else(|_| {
            unsafe { int_21h_ah_09h_out_str(b"Cannot read code page file.\r\n$".as_ptr()); }
            exit(1);
        }).ax_read;
        unsafe { int_21h_ah_09h_out_str(b"read\r\n$".as_ptr()); }
        if read == 0 { break; }
        code_page_buf = &mut code_page_buf[read as usize ..];
    }
    unsafe { int_21h_ah_09h_out_str(b"OK\r\n$".as_ptr()); }
    if !code_page_buf.is_empty() {
        unsafe { int_21h_ah_09h_out_str(b"Invalid code page file: too small.\r\n$".as_ptr()); }
        exit(1);
    }
    let code_page: &CodePage = unsafe { &*(code_page_memory.as_ptr() as *const CodePage) };
    exit(0);
}
