# C/C++ API

The Megaton C/C++ library offers tools for accessing the state of runtime modules, as well as for patching and hooking. A significant portion of the systems for patching and hooking has been derived from [exlaunch](https://github.com/shadowninja108/exlaunch).

## Prelude Includes
Include the `<megaton/prelude.h>` header to include common types and macros.
This is sometimes unnecessary if you are including other `megaton` API headers.

## Private Headers
Headers from `megaton/__priv` are internal headers and should not be included
in your project. The functionality is exposed through the public headers,
which provide more ergonomic and secure APIs.

## Naming Convention
The megaton public API has the following naming convention:
- `PascalCase` for structs and classes
- `snake_case` for functions
- `snake_case_` for macros (with trailing underscore)
