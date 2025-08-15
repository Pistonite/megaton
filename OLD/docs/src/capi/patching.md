# Patching in C++

The C++ Patching API provides a stream-like interface for patching
the main binary.

All patching should be done in the main function (i.e. `megaton_main`),
since editing the executable is not thread safe.

## Stream
The `main_stream` function constructs a patching stream starting
at an offset from the main binary. The `<<` operator is used
to patch the instructions. The write head will automatically advance
by the number of instructions written.

```cpp
#include <megaton/patch.h>

void patch() {
    megaton::patch::main_stream(0x02345678)
        << 0x1E2E1008  // fmov S8, #1.0
        << 0xBD045BEC; // str, S12, [SP, #0x458]
}
```

```admonish note
The `exl::armv8` types to generate instructions with CPP classes
are deprecated. They will be replaced with something more ergonomic
in the future.

For now, you can use tools like https://armconverter.com to generate
the instructions. Make sure to get the correct endianness.
```

The stream implements the RAII pattern. The cache will be flushed
when the stream is destroyed or goes out of scope.

## Branching
Use the `megaton::patch::b` and `megaton::patch::bl` functions
to create branch instructions to a function pointer. The relative 
offset is calculated automatically.

```cpp
#include <megaton/patch.h>

void my_func() {
    // Your code here
}

void patch() {
    megaton::patch::main_stream(0x02345678)
        << megaton::patch::b(my_func);
}
```

## Skip
Use `megaton::patch::skip(n)` to skip `n` instructions (i.e. advance
the write head without changing the instruction).

```cpp
#include <megaton/patch.h>

void patch() {
    megaton::patch::main_stream(0x02345678)
        << 0x1E2E1008  // fmov S8, #1.0
        << megaton::patch::skip(2) // skip 2 instructions
        << 0xBD045BEC; // str, S12, [SP, #0x458]
}
```

## Repeat
Use `megaton::patch::repeat(n, insn)` to repeated write an instruction `n` times.

```cpp
#include <megaton/patch.h>

void patch() {
    megaton::patch::main_stream(0x02345678)
        // write 3 nops
        << megaton::patch::repeat(3, exl::armv8::inst::Nop());
}
```
