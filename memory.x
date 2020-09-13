MEMORY
{
  /* NOTE: https://docs.platformio.org/en/latest/boards/ststm32/genericSTM32F407VET6.html */
  /* Generic doesn't seem to have specified amount of memory */

  /* NOTE K = KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
