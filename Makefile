%.ld.out: %.ld
	cat $< | clang-21 -E -CC - > $@
	sed -i .bak "s/\"\"//g" $@
	rm $@.bak

.PHONY: FORCE

test-%.elf: %.ld.out FORCE
	printf ".section \".lace.start\"\n.globl _start\n_start:\n" | arm-none-eabi-as -c -o $@.o
	ld.lld -T$< $@.o -o $@
	arm-none-eabi-readelf -a $@
	rm -f $@.o $@ $<.bak

clean:
	rm bcm2711.ld.out

FORCE: ;
