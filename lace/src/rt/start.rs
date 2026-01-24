//! Kernel entry point.
//!
//! After boot, will perform some initial setup before calling a function on core 0:
//! ```rs
//! #[unsafe(no_mangle)]
//! extern "C" fn _lace_main() -> ! { .. }
//! ```
//! Each other core will spin until [`wake_core_and_call`] is called for that core.

use core::arch::aarch64::{__dsb, __sev, SY};
use core::arch::naked_asm;
use core::cell::SyncUnsafeCell;
use core::marker::FnPtr;

const CORE_COUNT: usize = 4usize;

static INIT_CORE_SPIN: [SyncUnsafeCell<usize>; CORE_COUNT] =
    [const { SyncUnsafeCell::new(0) }; CORE_COUNT];

/// This is run on _all_ cores when the system starts. Handles various bits of low-level
/// initialization. Copied from Raspberry Pi's ripoff (armstub8) of ARM's BL31.
///
/// All cores except core0 will be sent into spinloops, from which they can be recovered using
/// [`wake_core_and_call`]. Core 0 will instead be sent to [`core0_start`].
#[unsafe(link_section = ".lace.start")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn _start() -> ! {
    naked_asm!(
    // /docs/bcm2836-peripherals.pdf, p.3-4: "divider = 2^31 / prescaler_value", so this gives us
    // divider ratio of 1.
    r#"
        ldr x0, ={local_control}
        str wzr, [x0]
        mov w1, #0x80000000
        str w1, [x0, #({local_prescaler} - {local_control})]
    "#,
    // Set up L2CTLR_EL1 - not totally sure what this is doing. It seems to be something about
    // setting up the L2 cache?
    //  - bits 2:0 are L2 ram latency, b010 is 3 cycles
    //  - bit 5 is L2 data ram setup, b1 is 1 cycle
    r#"
        mrs x0, s3_1_c11_c0_2
        mov x1, #0x22
        orr x0, x0, x1
        msr s3_1_c11_c0_2, x0
    "#,
    // Timer Counter Frequency - set to 54MHz; I think this is coming from an external crystal?
    r#"
        ldr x0, ={osc_freq}
        msr cntfrq_el0, x0
    "#,
    // Timer Virtual Offset
    "msr cntvoff_el2, xzr",
    // Architectural Feature Trap:
    //  - [12]ESM: trap execution of SME instructions, SVE inst. when no FEAT_SVE, SVE when in
    //             Streaming SVE mode, SMCR_ELx, SMPRI_EL1, SMPRIMAP_EL2, SVCR to EL3
    //  - [8]EZ: trap exception of SVE instructions when not in Streaming SVE mode, ZCR_ELx to EL3
    "msr cptr_el3, xzr",
    // Enable a few different things; don't really understand all of this stuff tbh
    r#"
        mov x0, {scr_val}
        msr scr_el3, x0
    "#,
    // Access to L2 control registers in EL<3
    r#"
        mov x0, {actlr_val}
        msr actlr_el3, x0
    "#,
    // necessary for caches and MMU
    r#"
        mov x0, {cpuectlr_el1_smpen}
        msr s3_1_c15_c2_1, x0
    "#,
    // Spin non-primary cores on INIT_CORE_SPIN, and send core0 to `core0_start`
    r#"
        mrs x0, MPIDR_EL1
        and x0, x0, #3
        cbz x0, 2f
        adr x1, {spin}
    1:
        wfe
        ldr x2, [x1, x0, lsl #3]
        cbz x2, 1b
        b 3f
    2:
        ldr x2, ={core0_start}
    3:
        br x2
    "#,
        core0_start = sym core0_start,
        spin = sym INIT_CORE_SPIN,

        local_control = const 0xff80_0000usize,
        local_prescaler = const 0xff80_0008usize,

        osc_freq = const 54_000_000u64,
        scr_val = const {
            (1 << 10) // RW: next lower exception level is using AArch64
            | (0 << 9) // SIF: permit execution from non-secure memory
            | (1 << 8) // HCE: enable HVC
            | (1 << 7) // SMD: disable SMC
            | (0 << 6) // reserved: 0
            | (3 << 4) // reserved: 11
            | (0 << 3) // EA:
            | (0 << 2) // FIQ:
            | (0 << 1) // IRQ:
            | (1 << 0) // NS: since NSE: is 0, security state for <=EL2 is Non-Secure.
        },
        actlr_val = const {
            (1 << 6) // L2ACTLR: accessible from lower exception level
            | (1 << 5) // L2ECTLR: accessible from lower exception level
            | (1 << 4) // L2CTLR: accessible from lower exception level
            | (1 << 1) // CPUECTLR: accessible from lower exception level
            | (1 << 0) // CPUACTLR: accessible from lower exception level
        },
        cpuectlr_el1_smpen = const {
            0 |
            (1 << 6) // SMPEN: enable processor to receive icache and TLB maintenance operations
                     //        from other processors in cluster, must be set before enabling caches
                     //        and MMU, or performing any cache/TLB maintenance ops
        }
    )
}

/// This is run on core 0 when the system starts; it serves to initialize the processor state into
/// something that Rust code can run in.
///
/// At the moment, that consists of:
///  1. Initializing the stack
///  2. Clearing the BSS
#[unsafe(naked)]
extern "C" fn core0_start() -> ! {
    unsafe extern "C" {
        static __lace_stack_init: [u128; 0];
        static __lace_bss_start: [u64; 0];
        static __lace_bss_end: [u64; 0];
    }
    // TODO: faster memset for BSS clear
    //  - no FEAT_MOPS, unfortunately (that would require v8.7+ I think)
    naked_asm!(
    r#"
        ldr x1, ={stack_init}
        mov sp, x1

        ldr x1, ={bss_start}
        ldr x2, ={bss_end}
        sub x3, x2, x1
        cbz x3, 2f
    1:
        str xzr, [x1], #8
        sub x3, x3, #8
        cbnz x3, 1b
    2:
        ldr x1, ={kernel_entry}
        br x1
    "#,
        stack_init = sym __lace_stack_init,
        bss_start = sym __lace_bss_start,
        bss_end = sym __lace_bss_end,
        kernel_entry = sym core0_entry,
    )
}

unsafe extern "C" {
    #[doc = "Main function defined elsewhere in the executable"]
    fn _lace_main() -> !;
}

/// This is run on core 0 when the system has started and the processor has entered a state that can
/// run Rust code.
extern "C" fn core0_entry() -> ! {
    unsafe { _lace_main() }
}

/// Wakes core number `core` and has it execute `func`.
///
/// # Safety
///
/// This function MUST NOT be called more than once for each `core` number.
//
// Note: this is an overly strict precondition, but I'm scared of UB from possible write races, and
//       relaxing this precondition doesn't really give any benefits.
pub unsafe fn wake_core_and_call(core: usize, func: extern "C" fn() -> !) {
    assert!(
        core < CORE_COUNT,
        "wake_core_to: core number out of range: {core} >= {CORE_COUNT}"
    );
    let ptr = INIT_CORE_SPIN[core].get();
    let func_addr: usize = FnPtr::addr(func).addr();
    // SAFETY: Aligned, word-sized accesses are atomic, and the reader is also "atomic" in the
    //         same way. This won't race with other writes because of the function safety
    //         conditions.
    //         I'm honestly not sure whether this is UB, but since the reader is outside of Rust's
    //         AM, I think this is okay, otherwise all of MMIO would be unsound (probably?).
    unsafe { ptr.write_volatile(func_addr) };

    // /docs/armv8a.pdf D1-7144:
    // "Arm recommends that software includes a DSB instruction before any SEV instruction. DSB
    // instruction ensures that no instructions, including any SEV instructions, that appear in
    // program order after the DSB instruction, can execute until the DSB instruction has
    // completed."
    // SAFETY: Doesn't break any of Rust's safety invariants.
    unsafe { __dsb(SY) };
    // Wake up any PEs that were sleeping on WFEs in the spinloop.
    // SAFETY: Doesn't break any of Rust's safety invariants.
    unsafe { __sev() };
}
