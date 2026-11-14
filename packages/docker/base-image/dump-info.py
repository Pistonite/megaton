# this script runs in the base-image to dump info needed
# for caching in the final image
import subprocess
import json

def dump_gcc_version():
    raw = (subprocess.run(["gcc", "--version"], capture_output=True, text=True, check=True)).stdout
    line = raw.splitlines()[0].strip()
    i = line.rfind(" ")
    if i < 0:
        return line
    return line[i+1:].strip()


def dump_clang_version():
    raw = (subprocess.run(["clang", "--version"], capture_output=True, text=True, check=True)).stdout
    line = raw.splitlines()[0].removeprefix("Debian clang version").strip()
    i = line.find(" ")
    if i < 0:
        return line
    return line[:i].strip()


def dump_bindgen_version():
    raw = (subprocess.run(["bindgen", "--version"], capture_output=True, text=True, check=True)).stdout
    line = raw.splitlines()[0].removeprefix("bindgen").strip()
    return line


def dump_rustc_version():
    raw = (subprocess.run(["rustc", "-vV"], capture_output=True, text=True, check=True)).stdout
    line = raw.splitlines()[0].strip().removeprefix("rustc").strip()
    i = line.find(" ")
    if i < 0:
        return line
    return line[:i].strip()
    

info = {
    "gcc_version": dump_gcc_version(),
    "clang_version": dump_clang_version(),
    "bindgen_version": dump_bindgen_version(),
    "rustc_version": dump_rustc_version(),
}

print(json.dumps(info, indent=4))
