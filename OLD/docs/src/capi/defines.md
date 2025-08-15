# Defines

In your project C/C++ sources, you can use these defines provided by `megaton` build tool.

```admonish tip
If the defines are not resolved correctly by your IDE/LSP server, try `megaton build --compdb`
to rebuild the compile database and restart your IDE/LSP server.
```

```admonish tip
Most of the defines can be accessed through a C++ API instead of macros.
```

## `MEGART_NX_MODULE_NAME`
String literal containing the name of the module, for example `"my_module"`.

## `MEGART_NX_MODULE_NAME_LEN`
Length of the module name, for example `8`.

## `MEGART_TITLE_ID`
Title ID of the module as a numeric literal, for example `0x0100000000000000`.

```admonish warning
The actual flag passed to compiler is the value in decimal, not hex.
```

## `MEGART_TITLE_ID_HEX`
Hex string literal of the title ID, for example `"0100000000000000"`.
