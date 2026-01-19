#![no_std]
#![no_main]
#![feature(stdarch_arm_barrier)]

use core::arch::aarch64::__dsb;

use tock_registers::Write as _;

core::arch::global_asm!(
    r#"
.section ".lace.start", "ax"
.globl _start
_start:
    mrs x0, MPIDR_EL1
    and x0, x0, #3
    cbz x0, 2f
    adr x1, _spin0
1:
    wfe
    ldr x2, [x1, x0, lsl #3]
    cbz x2, 1b
    b 3f
2:
    ldr x2, ={cpu0_start}
3:
    br x2

.globl _spin0, _spin1, _spin2, _spin3
_spin0: .quad 0
_spin1: .quad 0
_spin2: .quad 0
_spin3: .quad 0
"#,
    cpu0_start = sym cpu0_start2,
);

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub extern "C" fn cpu0_start2() -> ! {
    unsafe extern "C" {
        static __lace_stack_init: [u128; 0];
        static __lace_bss_start: [u64; 0];
        static __lace_bss_end: [u64; 0];
    }
    core::arch::naked_asm!(
        r#"
            // Initialize the stack
            ldr x1, ={stack_init}
            mov sp, x1

            // Initialize the BSS
            ldr x1, ={bss_start}
            ldr x2, ={bss_end}
            sub x3, x2, x1
            cbz x3, 2f
        1:
            str xzr, [x1], #8
            sub x3, x3, #8
            cbnz x3, 1b
        2:

            // Jump to the kernel
            b {kernel_entry}
        "#,
        stack_init = sym __lace_stack_init,
        bss_start = sym __lace_bss_start,
        bss_end = sym __lace_bss_end,
        kernel_entry = sym cpu0_start3,
    )
}

pub extern "C" fn cpu0_start3() -> ! {
    // let gpset1 = 0xfe20_0020 as *mut u32;
    // let gpclr1 = 0xfe20_002c as *mut u32;
    // loop {
    //     unsafe { gpset1.write_volatile(0xffff_ffff) }
    //     for _ in 0..0x100_0000 {
    //         unsafe { core::arch::asm!("") }
    //     }
    //     unsafe { gpclr1.write_volatile(0xffff_ffff) }
    //     for _ in 0..0x100_0000 {
    //         unsafe { core::arch::asm!("") }
    //     }
    // }
    let gpfsel1 = 0xfe20_0004 as *mut u32;
    unsafe { gpfsel1.write_volatile(gpfsel1.read_volatile() & !0o770000 | 0o440000) };

    unsafe { __dsb(core::arch::aarch64::SY) }

    // TODO: set up UART
    // TODO: print with UART

    // default: UARTCLK @ 48MHz, so baud rate is at most 3MHz by default
    // thus, we'll need to take the clock rate up to 96MHz

    loop {}
}

fn summon_uart(no: u8) -> Option<HwPL011> {
    if no == 1 || no >= 6 {
        None
    } else {
        Some(HwPL011::from_addr(0xfe201000 + 0x200 * usize::from(no)))
    }
}

tock_registers::register_bitfields![u32,
    pub Data [
        OE OFFSET(11) NUMBITS(1) [],
        BE OFFSET(10) NUMBITS(1) [],
        PE OFFSET(9) NUMBITS(1) [],
        FE OFFSET(8) NUMBITS(1) [],
        DATA OFFSET(0) NUMBITS(8) [],
    ],
    pub ReceiveStatusErrorClear [
        OE 3,
        BE 2,
        PE 1,
        FE 0,
    ],
    pub Flag [
        RI 8,
        TXFE 7,
        RXFF 6,
        TXFF 5,
        RXFE 4,
        BUSY 3,
        CTS 0,
    ],
    // Baud rate divisor calculation:
    //
    //  BAUDDIV = FUARTCLK / (16 * Baud Rate)
    pub IntegerBaudRateDivisor [
        IBRD OFFSET(0) NUMBITS(16) [],
    ],
    pub FractionalBaudRateDivisor [
        FBRD OFFSET(0) NUMBITS(6) [],
    ],
    pub LineControl [
        SPS OFFSET(7) NUMBITS(1) [],
        WLEN OFFSET(5) NUMBITS(2) [
            _8 = 0b11,
            _7 = 0b10,
            _6 = 0b01,
            _5 = 0b00,
        ],
        FEN OFFSET(4) NUMBITS(1) [],
        STP2 OFFSET(3) NUMBITS(1) [],
        EPS OFFSET(2) NUMBITS(1) [
            Odd = 0,
            Even = 1,
        ],
        PEN OFFSET(1) NUMBITS(1) [],
        BRK OFFSET(0) NUMBITS(1) [],
    ],
    pub Control [
        CTSEN 15,
        RTSEN 14,
        RTS 11,
        RXE 9,
        TXE 8,
        LBE 7,
        UARTEN 0,
    ],
    pub InterruptFifoLevelSelect [
        RXIFPSEL OFFSET(9) NUMBITS(3) [],
        TXIFPSEL OFFSET(6) NUMBITS(3) [],
        RXIFLSEL OFFSET(3) NUMBITS(3) [],
        TXIFLSEL OFFSET(0) NUMBITS(3) [],
    ],
    pub InterruptMaskSetClear [
        OEIM 10,
        BEIM 9,
        PEIM 8,
        FEIM 7,
        RTIM 6,
        TXIM 5,
        RXIM 4,
        CTSMIM 1,
    ],
    pub InterruptStatus [
        OERIS 10,
        BERIS 9,
        PERIS 8,
        FERIS 7,
        RTRIS 6,
        TXRIS 5,
        RXRIS 4,
        CTSRMIS 1,
    ],
    pub InterruptClear [
        OEIC 10,
        BEIC 9,
        PEIC 8,
        FEIC 7,
        RTIC 6,
        TXIC 5,
        RXIC 4,
        CTSMIC 1,
    ],
    pub DmaControl [
        DMAONERR 2,
        TXDMAE 1,
        RXDMAE 0,
    ],
];
tock_registers::peripheral! {
    #[real(HwPL011)]
    pub PL011 {
        0x00 => dr : Data::Register { Read, Write },
        0x04 => rsrecr : ReceiveStatusErrorClear::Register { Read, Write },
        0x18 => fr : Flag::Register { Read },
        0x24 => ibrd : IntegerBaudRateDivisor::Register { Read, Write },
        0x28 => fbrd : FractionalBaudRateDivisor::Register { Read, Write },
        0x2c => lcrh : LineControl::Register { Read, Write },
        0x30 => cr : Control::Register { Read, Write },
        0x34 => ifls : InterruptFifoLevelSelect::Register { Read, Write },
        0x38 => imsc : InterruptMaskSetClear::Register { Read, Write },
        0x3c => ris : InterruptStatus::Register { Read },
        0x40 => mis : InterruptStatus::Register { Read },
        0x44 => icr : InterruptClear::Register { Write },
        0x48 => dmacr : DmaControl::Register { Read, Write },
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
