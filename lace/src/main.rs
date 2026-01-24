#![no_std]
#![no_main]
#![feature(stdarch_arm_barrier, stdarch_arm_hints)]
#![feature(sync_unsafe_cell)]
#![feature(fn_ptr_trait)]
#![feature(likely_unlikely)]

use core::{arch::aarch64::__dsb, ptr::NonNull};

pub mod rt {
    pub mod start;
}
pub mod peri {
    pub mod sys_timer {
        use core::{
            arch::aarch64::{__dsb, SY},
            hint::likely,
            ops::Sub,
            time::Duration,
        };

        use tock_registers::Read as _;

        tock_registers::register_bitfields![u32,
            ControlStatus [
                M3 3,
                M2 2,
                M1 1,
                M0 0,
            ],
            CounterLow [ CNT OFFSET(0) NUMBITS(32) [] ],
            CounterHigh [ CNT OFFSET(0) NUMBITS(32) [] ],
            Compare [ CMP OFFSET(0) NUMBITS(32) [] ],
        ];
        tock_registers::peripheral! {
            #[real(HwSysTimer)]
            pub SysTimer {
                0x00 => cs: ControlStatus::Register { Read, Write },
                0x04 => clo: CounterLow::Register { Read },
                0x08 => chi: CounterHigh::Register { Read },
                0x0c => c0: Compare::Register { Read, Write },
                0x10 => c1: Compare::Register { Read, Write },
                0x14 => c2: Compare::Register { Read, Write },
                0x18 => c3: Compare::Register { Read, Write }
            }
        }
        pub fn floating_time_us() -> u64 {
            unsafe { __dsb(SY) };
            let sys_timer = HwSysTimer::from_addr(0xfe00_3000);
            let mut hi = sys_timer.chi().read();
            let ret = loop {
                let lo = sys_timer.clo().read();
                let hi2 = sys_timer.chi().read();
                if likely(hi2 == hi) {
                    break u64::from(hi) << 32 | u64::from(lo);
                }
                hi = hi2;
            };
            unsafe { __dsb(SY) };
            ret
        }
        #[derive(Copy, Clone)]
        pub struct Instant(u64);
        impl Instant {
            pub fn now() -> Self {
                Self(floating_time_us())
            }
        }
        impl Sub for Instant {
            type Output = core::time::Duration;

            fn sub(self, rhs: Self) -> Self::Output {
                core::time::Duration::from_micros(self.0.wrapping_sub(rhs.0))
            }
        }

        pub fn delay(duration: Duration) {
            let now = Instant::now();
            while (Instant::now() - now) < duration {}
        }
    }
}

// Called from rt::start::core0_entry
#[unsafe(no_mangle)]
extern "C" fn _lace_main() -> ! {
    let gpfsel1 = 0xfe20_0004 as *mut u32;
    unsafe { gpfsel1.write_volatile(gpfsel1.read_volatile() & !0o770000 | 0o440000) };
    unsafe { __dsb(core::arch::aarch64::SY) }
    let uart_pointer = unsafe {
        arm_pl011_uart::UniqueMmioPointer::new(NonNull::new(0xfe20_1000 as *mut _).unwrap())
    };
    let mut uart = arm_pl011_uart::Uart::new(uart_pointer);
    let _ = uart.enable(
        arm_pl011_uart::LineConfig {
            data_bits: arm_pl011_uart::DataBits::Bits8,
            parity: arm_pl011_uart::Parity::None,
            stop_bits: arm_pl011_uart::StopBits::One,
        },
        6_000_000,
        96_000_000, // set this up with init_uart_clock in config.txt
    );
    // okay, can now panic

    // use core::fmt::Write as _;
    // uart.write_str("Hello world from EL3 on the RPi4!\n");
    // #[derive(Debug)]
    // #[repr(C)]
    // struct ArmStubWords {
    //     /// This word is zeroed out by the firmware after it reads it.
    //     magic: u32,
    //     stub_version: u32,
    //     /// The address (32-bit) of the DTB the firmware loaded
    //     device_tree_addr32: u32,
    //     /// The address (32-bit) of the kernel the firmware loaded
    //     kernel_start_addr32: u32,
    // }
    // unsafe extern "C" {
    //     static __lace_safe_stub_words: ArmStubWords;
    // }
    // writeln!(uart, "safe stub words: {:x?}", unsafe {
    //     &__lace_safe_stub_words
    // });

    loop {
        core::hint::spin_loop()
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    use core::fmt::Write as _;

    // TODO(mc):
    //  - check if we're in an interrupt
    //  - disable interrupts if not, and drain out the (console) UART buffers

    crate::peri::sys_timer::delay(core::time::Duration::from_secs(1));

    // Fairly simple panic handler:
    //  - print out where the panic occurred
    //  - print out the panic message
    //  - reboot the SoC

    let uart_pointer = unsafe {
        arm_pl011_uart::UniqueMmioPointer::new(NonNull::new(0xfe20_1000 as *mut _).unwrap())
    };
    let mut uart = arm_pl011_uart::Uart::new(uart_pointer);

    if let Some(location) = info.location() {
        let _ = writeln!(
            uart,
            "panic occurred in file '{}' at line '{}:'",
            location.file(),
            location.line()
        );
    } else {
        // In the nightly at the time of writing, this branch is not reachable, but it's
        // included for completeness.
        let _ = writeln!(uart, "panic occurred but can't get location information!");
    }
    let _ = writeln!(uart, "{}", info.message());

    // crate::rt::reboot();
    loop {}
}
