# Tutorial

This section covers setting up a new Megaton project and compiling it
using the Megaton build tool.

At this point, you should installed Megaton by following the
[installation guide](../install.html).

```
# Ensure that megaton is installed and is using the version you expect
$ megaton --help
 __    __ ______ ______ ______ ______ ______ __   __
/\ "-./  \\  ___\\  ___\\  __ \\__  _\\  __ \\ "-.\ \
\ \ \-./\ \\  __\ \ \__ \\  __ \_/\ \/ \ \/\ \\ \-.  \
 \ \_\ \ \_\\_____\\_____\\_\ \_\\ \_\\ \_____\\_\\"\_\
  \/_/  \/_//_____//_____//_/\/_/ \/_/ \/_____//_/ \/_/

Megaton Build Tool

Usage: megaton <COMMAND>

Commands:
  version    Print the version and build information
  build      Compile and link the megaton project
  toolchain  Manage the custom `megaton` Rust toolchain
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --version  Print the version
  -h, --help     Print help

$ megaton version
megaton v0.1.0 (5939be53)
```
