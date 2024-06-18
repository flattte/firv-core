#![no_std]
#![no_main]

use core::{
    arch::{asm, global_asm},
    fmt::{self, Write},
    panic::PanicInfo,
};

global_asm!(include_str!("../res/entry.s"));
/// A panic handler is required in Rust, this is probably the most basic one possible
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut uart = Uart;
    writeln!(uart, "panic!");
    writeln!(uart, "{}", info).ok();

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
static mut UART_: Uart = Uart;

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

fn put_bytes(bytes: &[u8]) {
    for byte in bytes {
        unsafe { mmio_write(UART, *byte) }
    }
}

fn putc(c: char) -> () {
    unsafe {
        mmio_write(UART, c as u8);
        mmio_write(UART, '\n');
    }
}

#[firv_harden]
#[no_mangle]
extern "C" fn sw_f(a: i32, b: i32) -> i32 {
    let x = a + b;
    let y = a - b;

    return x * y;
}

#[repr(C)]
struct Ab {
    a: i32,
    b: i32,
}

#[firv_harden]
#[no_mangle]
extern "C" fn sw_f2(ab: Ab) -> i32 {
    let x = ab.a + ab.b;
    let y = ab.a - ab.b;

    return x * y;
}

#[derive(Debug)]
struct Cd {
    c: u64,
    d: u64,
}

#[firv_harden]
#[no_mangle]
fn sw_f3(a: u64, b: u64) -> Cd {
    let x = a + b;
    let y = a - b;

    writeln!(unsafe { UART_ }, "hello there").unwrap();

    return Cd { c: x, d: y };
}

#[derive(Debug)]
struct Vec2 {
    x: u32,
    y: u32,
}

#[firv_harden]
#[no_mangle]
// #[firv_harden]
fn make_vec(x: u32, y: u32) -> Vec2 {
    let x = x + 40;
    let y = y + 30;
    Vec2 { x, y }
}

#[firv_harden]
#[no_mangle]
#[inline(never)]
fn side_effect(a: i32, b: i32) -> i32 {
    unsafe {
        mmio_write(UART, 'a' as u8);
        mmio_write(UART, '\n');
    };
    a + b
}

#[firv_harden]
#[no_mangle]
#[inline(never)]
fn foo2(a: i32, b: i32) -> i32 {
    // let mut a = a;
    writeln!(unsafe { UART_ }, "foo foo  2");
    // unsafe {
    //     for i in 0..(b as usize * (*c)) {
    //         mmio_write((&mut a as *mut i32) as usize, i);
    //     }
    // }
    a + b
}

#[firv_harden]
#[no_mangle]
fn codegen_parity(a: i32, b: i32) -> i32 {
    a + b
}

#[firv_harden]
#[no_mangle]
fn side_effect_one(a: i32, b: i32) -> i32 {
    let ret = a + b;
    writeln!(unsafe { UART_ }, "side_effect_one return value: {:?}", ret);
    ret
}

#[derive(Debug)]
struct C {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct S {
    n: i32,
    c: C,
}

#[firv_harden]
#[no_mangle]
fn side_effect_two(n: i32, x: i32, y: i32) -> S {
    let ret = S { n, c: C { x, y } };
    writeln!(unsafe { UART_ }, "side_effect_one return value: {:?}", ret);
    ret
}

#[no_mangle]
extern "C" fn main() -> () {
    let _ = side_effect_two(1, 2, 3);
}
