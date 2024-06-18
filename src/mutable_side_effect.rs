#![no_std]
#![no_main]
#![feature(panic_info_message)]
use core::{
    arch::global_asm,
    fmt::{self, Write},
    panic::PanicInfo,
};

global_asm!(include_str!("../res/entry.s"));
/// A panic handler is required in Rust, this is probably the most basic one possible
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    write!(Uart, "Aborting: ").ok();
    if let Some(p) = info.location() {
        writeln!(
            Uart,
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        )
        .ok();
    } else {
        writeln!(Uart, "no information available.").ok();
    }

    fn inner_exit() {
        exit();
    }
    inner_exit();
    loop {}
}

const UART: usize = 0x10000000;
const LSR: usize = UART + 5;
const SYSCON: usize = 0x100000;
const LSR_TX_IDLE: u8 = 1 << 5;
const SHUTDOWN: u32 = 0x5555;

#[derive(Clone, Copy)]
struct Uart;

impl Uart {
    const BASE_ADDRESS: *mut u8 = 0x10000000 as *mut u8;
    pub fn write_byte(&self, byte: u8) {
        unsafe {
            loop {
                if (LSR as *const u8).read_volatile() & LSR_TX_IDLE > 0 {
                    break;
                }
            }
            Self::BASE_ADDRESS.write_volatile(byte);
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            self.write_byte(*c);
        }
        Ok(())
    }
}

unsafe fn mmio_write<T>(addr: usize, value: T) {
    let reg = addr as *mut T;

    reg.write_volatile(value);
}

fn exit() {
    unsafe { mmio_write(SYSCON, SHUTDOWN as u32) }
}

fn putc(c: char) -> () {
    unsafe {
        mmio_write(UART, c as u8);
        mmio_write(UART, '\n');
    }
}

const KEY: u64 = 3;

#[derive(Debug)]
struct Secret {
    a: u64,
    b: u64,
}

#[firv_harden]
#[no_mangle]
#[inline(never)]
fn important_check(a: u64, fault: u64) -> Secret {
    unsafe {
        (fault as *mut u64).write_volatile(*(fault as *mut u64) + 1);
    }
    let key_local = a + unsafe { (fault as *mut u64).read_volatile() };
    let mut secret = Secret { a: 0, b: 0 };

    if key_local == KEY {
        secret.a = 100;
        secret.b = 200;
    }

    secret
}

#[no_mangle]
extern "C" fn main() -> () {
    let mut rr: u64 = 0;
    let ptr = &mut rr as *mut u64;
    unsafe { ptr.write_volatile(0) };
    let secret = important_check(1, ptr as u64);
    writeln!(Uart, "{:?}", secret);
    exit()
}
