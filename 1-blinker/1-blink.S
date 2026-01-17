.arch armv8-a

.equ gpio_base, 0xfe200000
//.equ local_control, 0xff800000
//.equ local_prescaler, 0xff80000c
//.equ gic_distb, 0xff841000
//.equ gic_cpub, 0xff842000
//.equ gpio_base, 0x47e200000
//.equ local_control, 0x47f800000
//.equ local_prescaler, 0x47f80000c
//.equ gic_distb, 0x47f841000
//.equ gic_cpub, 0x47f842000


.equ osc_freq, 54000000
.equ scr_val, 0b10110110001
.equ actlr_val, 0b1110011
.equ cpuectlr_el1_smpen, 0b1000000
.equ spsr_el3_val, 0x3c9

.equ gicc_ctrlr, 0
.equ gicc_pmr, 4
.equ it_nr, 8
.equ gicd_ctrlr, 0
.equ gicd_igroupr, 0x80

.extern __lace_stack_init
.extern __lace_bss_start
.extern __lace_bss_end

.section ".lace.start", "ax"
.globl _start
_start:
/*
        ldr x0, =local_control
        str wzr, [x0]
        mov w1, 0x80000000
        str w1, [x0, (local_prescaler - local_control)]

        mrs x0, s3_1_c11_c0_2
        mov x1, 0x22
        orr x0, x0, x1
        msr s3_1_c11_c0_2, x0

        ldr x0, =osc_freq
        msr cntfrq_el0, x0

        msr cntvoff_el2, xzr

        msr cptr_el3, xzr

        ldr x0, =scr_val
        msr scr_el3, x0

        ldr x0, =actlr_val
        msr actlr_el3, x0

        ldr x0, =cpuectlr_el1_smpen
        msr s3_1_c15_c2_1, x0

        ldr x0, =0x30c50830
        msr sctlr_el2, x0

        ldr x0, =spsr_el3_val
        msr spsr_el3, x0
        adr x0, in_el2
        msr elr_el3, x0
        eret
*/

in_el2:
        mrs x6, mpidr_el1
        and x6, x6, 3
        cbz x6, primary_cpu

secondary_spin:
        wfe
        b secondary_spin

primary_cpu:
        ldr  x1, =__lace_stack_init
        mov sp, x1

        ldr x1, =__lace_bss_start
        ldr x2, =__lace_bss_end
        sub x3, x2, x1
        cbz x3, bssclear
zbss:   str xzr, [x1], #8
        sub x3, x3, #8
        cbnz x3, zbss
        
bssclear:
        bl notmain
        b secondary_spin
