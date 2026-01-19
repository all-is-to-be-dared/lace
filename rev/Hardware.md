# Documentation

BCM2711 inherits from BCM2710 inherits from BCM2836 inherits from BCM2835 / BCM2708

Documents:
 - docs/cortex_a72.pdf
 - docs/armv8a.pdf
 - docs/bcm2711.pdf
 - docs/bcm2836.pdf
 - docs/bcm2835.pdf
 - https://github.com/hermanhermitage/videocoreiv (/wiki)
 - https://github.com/christinaa/rpi-open-firmware

# Memory

### Gross Memory Layout

```
0x0_0000_0000 : 0x0_4000_0000-$gpu_mem   ARM DRAM 0
0x0_4000_0000 : (purchase option)        ARM DRAM 1
0x6_0000_0000 : 0x6_4000_0000            PCI
```

In the middle is the section of memory reserved by the VC.
Note that this is _not_ the peripherals block.

The location of the peripherals depends on configuration: there is a configuration option,
`arm_peri_high`, that controls whether the Broadcom/ARM peripherals are mapped in at `FC00_0000`
or `4_7C00_0000`. The default is `FC00_0000`, for 32-bit compat reasons, but `4_7C00_0000` allows
all of DRAM to be mapped on devices with 4GiB of DRAM (while LPAE would allow for its use on 32-bit
kernels, it would apparently complicate the early stage bootloader, so the Pi folks leave it be).


### Memory Hierarchy

```
  DRAM
    |
 BCM2711 | L2
    |
Cortex-A72 | L2
             |
             L1 L1 L1 L1
```
As far as I can tell, this is the way the cache hierarchy on the pi4 works: L1s per core on the A72
and an L2 on the A72, which then goes to the VC's L2, and subsequently the DRAM.

> [!TODO]
> Haven't actually figured out the sizes of the different pieces yet.

In this case, the VC's L2 is almost a kind of L3?

### Clock Hierarchy

This is mostly extrapolated from reverse engineering on prior SoCs in the 2708-series, but it seems
like clocks live in the CM block at `E10_1000:E10_2000`.
These are the lower-level clocks that feed off the PLLs, however, and only have prescaler
functionality.

It is _believed_ that the PLLs are controlled by the A2W region at `E10_2000`, and the PLLx
multiplier registers are known, at the very least.