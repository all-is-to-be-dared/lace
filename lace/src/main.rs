#![no_std]
#![no_main]

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
    let gpset1 = 0xfe20_0020 as *mut u32;
    let gpclr1 = 0xfe20_002c as *mut u32;
    loop {
        unsafe { gpset1.write_volatile(0xffff_ffff) }
        for _ in 0..0x100_0000 {
            unsafe { core::arch::asm!("") }
        }
        unsafe { gpclr1.write_volatile(0xffff_ffff) }
        for _ in 0..0x100_0000 {
            unsafe { core::arch::asm!("") }
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
