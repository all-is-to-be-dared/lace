enum {
  peri_base = 0xfc000000,

  gpio_base = peri_base + 0x02200000,
  gpset1 = gpio_base + 0x20,
  gpclr1 = gpio_base + 0x2c,
};

void cput(long a, unsigned v) { *(volatile unsigned *)a = v; }
unsigned cget(long a) { return *(volatile unsigned *)a; }

__attribute__((noreturn)) void notmain(void) {
#define NITER 0x2000000
  int i;
  while (1) {
    cput(gpset1, 0xffffffff);
    for (i = 0; i < NITER; i++)
      __asm__("" :::);
    cput(gpclr1, 0xffffffff);
    for (i = 0; i < NITER; i++)
      __asm__("" :::);
  }
}
