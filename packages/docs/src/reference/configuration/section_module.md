
In a Megaton project, the module is the NSO object that is ultimately produced and loaded alongside
the game. It is also sometimes synonymus with project.

Example:
```toml
[module]
name = "my-mod"
title-id = 0x01007ef00011e000
```

```admonish tip
This root of a particular project is the directory that contains the config (Megaton.toml).
For all values that determine a path, unless otherwise specified,
the path is relative to the project root.

For each key, if a default value/behavior is not specified, the entry is required in the config.
```

### Key: `module.name` (`string`)
The name of the module, i.e., your mod's name. The final binary will use this name.

Restrictions: Cannot be `""` or `"lib"`. Only alphanumeric characters, `-`, and `_` are allowed.

### Key: `module.title-id` (`integer`)
The title ID for the targeted game. Needed to generate the NPDM file. 
(Note this needs to be an integer in the config, not a hex string)

### Key: `module.target` (string)
The path to the directory for the Megaton build tool to emit build artifacts.
All the library and generated artifacts will be placed here under a subdirectory `megaton`.
(For example if `module.target = "foo/bar"`, then all megaton's output will be at `foo/bar/megaton/`.)
See [Output Directory](../output_formats/output_directory.md) for the structure of the directory.

Default: `"target"`

### Key: `module.compdb` (string)
A database of compiler commands to be used for clangd integration.
Megaton will work with existing `compile_commands.json` (included ones generated from another Megaton project). This allows multiple Megaton projects in the same monorepo to share the same `compile_commands.json`
to make LSP integration easier. However, other tools like CMake could still override megaton's entries

Default: `"compile_commands.json"`
