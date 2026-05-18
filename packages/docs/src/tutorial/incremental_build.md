# Incremental Build

The Megaton build tool uses an incremental build system which means that
compilation artifacts are only regenerated if necessary. For building rust
code, Megaton uses cargo, which already has incremental build support. For
compiling C/C++/Assembly sources, Megaton determines if a source file must be
recompiled or if the output NSO file must be relinked.

## Intermediate artifacts

Following a normal build, the target directory will contain several intermediate
artifacts that Megaton uses to determine if recompilation/relinking is needed.

The main files are `compiledb.cache` and `linkcmd.cache`. These files contain
records of the compilation/link jobs that were used for the most recent version
of a given `.o` or ELF file.

For each compiled source, a `.o` and `.d` file will be generated in the `o/` 
directory pertaining to the profile and module. The `.o` file will be eventually
linked to create the ELF and NSO files, and the `.d` file is used by the build
tool to determine which dependencies must be checked to know if a source must
be recompiled. These filenames contain the hash of their path so that they
can be stored in a flat directory without name collisions.

The page [Output Directory (WIP)](./reference/output_formats/output_directory.html)
contains a full description of the contents and layout of the target directory.

## Incremental step checks

For a given source file, the following checks are made to determine if compilation
can be skipped. This check is run after the source is encountered and a command
is generated to compile the object if it is not longer up to date.
For assembly files, all checks relating to `.d` files are skipped as assembly
sources cannot have dependencies.
The checks are made in this order.

The file is considered up to date if:
- A record of the source being compiled exists in `compiledb.cache`
- The arguments for the pending compile command match the previous compile command
- A previously compiled `.o` file exists in the expected location
- A previously compiled `.d` file exists in the expected location
- The time of previous modification (mtime) for the source file is the same
  as the `.o` file's mtime
- The source's mtime is the same as the `.d` file's mtime
- The source's mtime is equal or newer than any of its dependencies
  (checked recursively by parsing the `.d` file)

The linked binary is considered up to date if:
- Cargo did not change the previously generated static lib
- None of the compilation tasks actually compiled anything
- The output ELF and NSO files exist
- The previously stored link command is the same as the impending link command
