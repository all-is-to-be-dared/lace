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

0x0_0000_0000 : 0x0_4000_0000-%gpu_mem   ARM DRAM 0
0x0_4000_0000 : ?                        ARM DRAM 1
0x6_0000_0000 : 0x6_4000_0000            PCI

In the middle is the section of memory reserved by the VC.

### Memory Hierarchy

 BCM2711 - L2
    |
Cortex-A72 - L2
             |
             L1 L1 L1 L1
