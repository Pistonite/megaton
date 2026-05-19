# Test Mod

Using the provided test mod is a simple and fast way to verify you have Megaton
configured correctly for your game.

## Prerequisites
This guide assumes the following:
### Megaton
Megaton and the Megaton toolchain should be installed. Refer to the appropriate
Megaton documentation for setup. Additionally, the (Megaton repository)[https://github.com/Pistonite/megaton]
should be cloned to your computer, as this is where the test mod is located
### Game
The Megaton test mod, and Megaton broadly, require the title id of your game
and a symbol file containing Nintendo SDK symbols. The test mod provides an example
title-id and SDK file for The Legend of Zelda: Breath of the Wild, but if you wish
to mod another game then you must provide these yourself. You will also need a (legal) copy
of BOTW or the game you wish to mod.
### Nintendo Switch
This guide will assume that you already have a modded Switch. If you don't have a
modding capable Switch or otherwise wish to use an emulator, you should be familiar
with how mods are installed for your emulator of choice.
## Modifying Megaton Config
The test mod assumes you will be modding BOTW. If you wish to mod another game,
the Megaton config will need to be modified accordingly. This file can be found at
`packages/test-mod/Megaton.toml` in the Megaton repository. You will need to modify the `title-id` field to
match the title id of your game. You will also need to replace `packages/test-mod/sdk.syms` with your
game's SDK symbols. The name of this file can be changed with the `symbols` field in `Megaton.toml`.
It can also be split into multiple files, so long as each one is added to the `symbols` field.
## Building
Building the test mod is quite simple. CD into the test mod directory and run the `megaton build` command,
if all finished successfully, then congrats! Megaton was able to build your test mod. You can now install it to
your Switch or emulator. If Megaton exited with a "Check failed" error caused by missing symbols,
verify that the missing symbols are present in your SDK symbols file.
## Installing
We will now install the test mod onto your modded Switch. If you instead wish to use
an emulator, you can follow the instructions provided by your emulator of choice.
The mod should be located at `packages/test-mod/target/none/TestMod/TestMod.nso`. If
the `TestMod` folder or `TestMod.nso` doesn't exist, check your `Megaton.toml`. The name of your
mod should correspond to the `module.name` field of `Megaton.toml`. TODO: Copying to Switch
