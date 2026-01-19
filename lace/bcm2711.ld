#include "cpp_helpers.inc"
AUTO_GENERATED_HEADER()

/* CONFIGURABLE VALUES ------------------------------------------------------------------------- */

/* u-boot : default load address is 0x100_0000 = 16M */
/* #define LACE_LOAD_ADDR 0x1000000 */
#define LACE_LOAD_ADDR 0x80000

/* armv8a : exception vector is 2K-aligned and 2K-size */
#define VECTOR_ALIGN 2K
#define VECTOR_LEN 2K

/* bcm2711 : gpu_mem defaults to 76M */
#define GPU_MEM 76M
/* erinna : my particular rpi4b has 2G memory */
#define TOTAL_MEM 2048M

/* FIXED VALUES -------------------------------------------------------------------------------- */

/* aarch64 : */
#define ALIGN_INST 4
#define ALIGN_WORD 8
/* abi : */
#define ALIGN_STACK 16

/* cortex-a72 : L1 and L2 have 64-byte fixed cache line size */
#define ALIGN_CACHE 64
/* cortex-a72 : L1 TLB is 4K/64K/1M, L2 TLB is 4K/64K/1M/16M */
#define ALIGN_TLB 4K

/* MEMORIES ------------------------------------------------------------------------------------ */

MEMORY {
  cpu_dram0 : ORIGIN = 0, LENGTH = (1024M - GPU_MEM)
  cpu_dram1 : ORIGIN = 1024M, LENGTH = (TOTAL_MEM - 1024M)
}

/* PROGRAM ENTRY ------------------------------------------------------------------------------- */

/* This is just to ensure that the linker will warn if no _start symbol is defined, since the
   linker has a rather elaborate set of fallbacks in the event that no ENTRY() is given or no
   _start is defined */
ENTRY(_start)
ASSERT(DEFINED(_start), "lace: _start MUST be defined");

/* SECTIONS ------------------------------------------------------------------------------------ */

SECTIONS {
  .text LACE_LOAD_ADDR (TYPE=SHT_PROGBITS) : {
    /* SAFETY: push is pre-indexed, so pushes to the stack will always be _below_ the initial stack
               pointer */
    __lace_stack_init = .;


    /* Mark the beginning of the executable */
    __lace_exec_start = .;


    /* This MUST contain the _start symbol */
    KEEP(*(.lace.start))
    . = ALIGN(ALIGN_INST);

    
    /* Next, make space for the vector table: we want to make sure that the vector table is able to
       jump quickly to all (?) of the .text section */
#define SUBVECTOR(BASE, LEVEL, OFFSET, NAME) \
                 . = __lace_vector_base + BASE + OFFSET; \
                 KEEP (*(.lace.vector.LEVEL.NAME)) \
                 ASSERT ((. - __lace_vector_base - BASE - OFFSET) < 0x080, "lace: .lace.vector."XSTR(LEVEL)XSTR(NAME)" is too long"); \
                 . = __lace_vector_base + BASE + OFFSET + 0x080;
#define VECTOR(BASE, LEVEL)    \
                 SUBVECTOR(BASE,LEVEL,0x000,synch) \
                 SUBVECTOR(BASE,LEVEL,0x080,irq) \
                 SUBVECTOR(BASE,LEVEL,0x100,fiq) \
                 SUBVECTOR(BASE,LEVEL,0x180,serror)
    . = ALIGN(VECTOR_ALIGN);
    __lace_vector_base = .;
    VECTOR(0x000,curr_el0)
    VECTOR(0x200,curr_elnz)
    VECTOR(0x400,lower64)
    VECTOR(0x600,lower32)
#undef VECTOR
#undef SUBVECTOR


    /* Finally, the normal contents of the text section: */
    *(.text*)


    /* Pad .text out to word alignment */
    . = ALIGN(ALIGN_WORD);
  } >cpu_dram0

  .rodata : ALIGN(ALIGN_TLB) {
    /* Read-only data */
    *(.rodata*)
    . = ALIGN(ALIGN_WORD);
  } >cpu_dram0
  
  .data : ALIGN(ALIGN_TLB) {
    /* Read + write data with a nonzero initial bit pattern */
    *(.data*)
    . = ALIGN(ALIGN_WORD);
  } >cpu_dram0

  .bss : ALIGN(ALIGN_TLB) {
    /* Read (+ write?) data with a zero initial bit pattern */
    __lace_bss_start = .;
    *(.bss*)
    *(COMMON)
    . = ALIGN(ALIGN_WORD);
    __lace_bss_end = .;


    /* Mark the end of the executable */
    __lace_exec_end = .;
  } >cpu_dram0
}

