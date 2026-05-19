# Check

The Megaton check system allows an additional layer of verification following
linking. The check will ensure that there are no undefined symbols or disallowed
instructions.

The check will run anytime the ELF is relinked, and if the check fails, the
ELF will not be converted into an NSO, and the build will fail.

For a full guide on configuring the check step, see the
[configuration page](../reference/configuration/section_check.html)

## Symbol check

The built mod could cause crashes or unexpected behavior if certain symbols
remain undefined in the final binary. This is especially true while the tool
is in active development and certain standard library features are not yet
supported. Megaton will use `objdump` to obtain a list of dynamic symbols.
Dynamic symbols are only considered as defined if an included symbol file
specified in `check.symbols`. If check reports that a particular syscall
or system function is undefined, ensure that your mod SDK symbol file
contains that symbol.

## Disallowed instructions

During Megaton library development, it may be convenient to disable certain
instructions to prevent crashes due to assembly instructions that are known
to crash due to limitations in Megaton library support.
