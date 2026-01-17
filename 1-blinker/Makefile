%.ld.out: %.ld
	cat $< | clang-21 -E -CC - > $@
	sed -i .bak "s/\"\"//g" $@
	rm $@.bak

test-%.elf: %.ld.out FORCE
	printf ".section \".lace.start\"\n.globl _start\n_start:\n" | aarch64-elf-as -c -o $@.o
	ld.lld -T$< $@.o -o $@
	aarch-elf-readelf -a $@
	rm -f $@.o $@ $<.bak

1-blink.elf: 1-blink.S 1-blink.c bcm2711.ld.out
	aarch64-elf-gcc -O3 1-blink.S 1-blink.c -Tbcm2711.ld.out -o $@ -ffreestanding -nostdlib -march=armv8-a -mcpu=cortex-a72

1-blink.bin: 1-blink.elf
	aarch64-elf-objcopy -O binary $< $@

run: 1-blink.bin
	mv $< /private/tftpboot/lace.bin

.PHONY: FORCE clean run
FORCE: ;

clean:
	rm -f bcm2711.ld.out 1-blink.elf 1-blink.bin

