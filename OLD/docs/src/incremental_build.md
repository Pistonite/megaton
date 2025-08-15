# Incremental Build

Megaton is optimized for incremental builds, i.e. the following cases:
- Rebuild after changing a source file
- Rebuild after creating/removing source files
- Rebuild after changing the build configuration

Megaton also uses GCC's deps file and OS timestamps to make sure the correct changes are detected
and rebuilt for incremental builds.

However, there are some other types of change that may require a full rebuild,
but isn't detected:
- Upgrading the toolchain (i.e. DevKitPro) or a tool used in the build
  - Run `megaton checkenv` and `megaton clean` to clear env cache and build cache
- Updating the `megaton` tool or library
  - Run `megaton build --lib` to rebuild the library
- Outputs/timestamps are manually changed
  - Run `megaton clean` to clear build cache


