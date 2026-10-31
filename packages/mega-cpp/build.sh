#!/usr/bin/env bash
set -euo pipefail

# This is a script to rebuild the megaton library objects on the field
# i.e. you can edit megaton source in the copy of megaton library unpacked
# by the megaton buildtool, then run this script to rebuild it.
#
# To override your changes, edit `.sha256` in the unpacked `lib` directory.
# The buildtool will re-unpack the library next time it runs.

cd "$(dirname "$0")"

if [ ! -f build/CMakeCache.txt ]; then
    echo 'configuring cmake'
    cmake -G Ninja -B build
fi

cmake --build build

rm -rf lib
mkdir -p lib/hermit

# copy rocrt objects
cp build/src/rocrt/CMakeFiles/megarocrt.dir/rocrt.cpp.obj lib/rocrt.o
cp build/src/rocrt/CMakeFiles/megarocrt.dir/rocrt0.s.obj lib/rocrt0.o

# copy hermit objects
hermit_dir=build/src/hermit/CMakeFiles/megahermit.dir
find "$hermit_dir" -name '*.cpp.obj' -print0 | while IFS= read -r -d '' obj; do
    rel="${obj#"$hermit_dir"/}"
    out="lib/hermit/${rel%.cpp.obj}.o"
    mkdir -p "$(dirname "$out")"
    cp "$obj" "$out"
done

# copy libmegacpp
cp build/src/megacpp/libmegacpp.a lib/libmegacpp.a

echo 'copied output files'
