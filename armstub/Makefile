CC=aarch64-elf-gcc
OBJCOPY=aarch64-elf-objcopy
ASFLAGS=-ffreestanding -nostartfiles -nostdlib -mcpu=cortex-a72 -march=armv8-a
LDFLAGS=-Wl,--section-start=.text=0

armstub8.elf: armstub8.S
	$(CC) $(ASFLAGS) $(LDFLAGS) -DGIC=1 -DBCM2711=1 -DHIGH_PERI=0 -o $@ $^
armstub8.bin: armstub8.elf
	$(OBJCOPY) -O binary $< $@
