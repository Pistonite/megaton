#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Megaton contributors
# * * * * *
# AI Generated

# Set up and patch the local Rust compiler source (rust-lang/rust) that the
# megaton toolchain is built from.
#
# The repo is owned by this script and lives at ../rust. Patches live next to
# this script and are named rust-<yyyy-mm-dd>-<revision>.patch, where <revision>
# is the 16-char commit the patch is based on. The latest patch by date is the
# one that gets used.
#
# The repo is kept in this "normalized" state:
#   main  - tracks origin/main, the latest upstream main
#   patch - local-only branch, exactly one commit ahead of <revision>, holding
#           the whole patch as a single commit
#
# Commands:
#   setup       clone the repo (or reset the existing one), update main, and
#               remake the patch branch from the latest patch file. having no
#               patch file at all is fine, the patch branch is just skipped.
#   make-patch  bake the current bootstrap change-id into bootstrap.megaton.toml
#               (as its own commit, if it changed), write a new patch file from
#               the current HEAD, diffed against origin/main, and update
#               BLESSED_COMMIT/BLESSED_VERSION in src/rust_toolchain.rs.
#
# Usage: python3 scripts/setup_rust.py <setup|make-patch>
# The usual flow is: run `setup`, hack on ../rust, commit, then run `make-patch`.

import argparse
import re
import shutil
import subprocess
import sys
from datetime import date
from pathlib import Path
from typing import NamedTuple

ROOT = Path(__file__).parent.parent
SCRIPTS_DIR = ROOT / "scripts"
RUST_DIR = ROOT / "rust"
TOOLCHAIN_RS = ROOT / "src" / "rust_toolchain.rs"

RUST_REPO = "https://github.com/rust-lang/rust"
PATCH_BRANCH = "patch"
# our own bootstrap config, in the rust repo, provided by the patch
BOOTSTRAP_TOML = "bootstrap.megaton.toml"
CHANGE_TRACKER_RS = "src/bootstrap/src/utils/change_tracker.rs"
REVISION_LEN = 16
PATCH_RE = re.compile(rf"^rust-(\d{{4}}-\d{{2}}-\d{{2}})-([0-9a-f]{{{REVISION_LEN}}})\.patch$")


def main() -> int:
    parser = argparse.ArgumentParser(description="set up/patch the local rust compiler source")
    parser.add_argument(
        "command",
        choices=["setup", "make-patch"],
        help="setup: normalize the repo. make-patch: save the current HEAD as a new patch",
    )
    args = parser.parse_args()
    try:
        if args.command == "setup":
            setup()
        else:
            make_patch()
    except Bail as e:
        error(e)
        return 1
    return 0


def setup() -> None:
    """Put the rust repo into the normalized state."""
    if is_git_repo(RUST_DIR):
        info("resetting the working tree")
        git("reset", "--hard")
    else:
        clone_rust_repo()
    # we check out revisions directly, no need for the wall of detached HEAD advice
    git("config", "--local", "advice.detachedHead", "false")

    info("updating main to the latest origin/main")
    git("fetch", "origin", "main", "--progress")
    git("checkout", "-B", "main", "origin/main")
    git("branch", "--quiet", "--set-upstream-to=origin/main", "main")

    patch = find_latest_patch()
    if patch is None:
        info("no patch file found, skipping making the patch branch")
        return
    make_patch_branch(patch)
    info("setup done!")


def make_patch() -> None:
    """Save the current HEAD as a new patch file, based on origin/main."""
    if not is_git_repo(RUST_DIR):
        raise Bail(f"'{RUST_DIR}' is not a git repo. run the setup command first")
    if is_dirty():
        raise Bail("the working tree is dirty. commit or discard the changes first")

    # note origin/main is intentionally not fetched here - the patch must be
    # based on the same revision the current HEAD was built on
    base = rev_parse("origin/main")
    head = rev_parse("HEAD")
    if base == head:
        raise Bail("HEAD is the same commit as origin/main, there is nothing to make a patch from")

    # this may add another commit on top, so HEAD has to be read again after
    update_change_id()
    head = rev_parse("HEAD")

    revision = base[:REVISION_LEN]
    today = date.today().isoformat()
    patch_name = f"rust-{today}-{revision}.patch"
    patch_path = SCRIPTS_DIR / patch_name
    if patch_path.exists():
        info(f"{val(patch_name)} exists, overriding!")

    info(f"making patch from {val(head[:REVISION_LEN])} based on {val(revision)}")
    diff = git_bytes("diff", "--binary", base, head)
    if not diff.strip():
        raise Bail("the diff against origin/main is empty")
    patch_path.write_bytes(diff)
    info(f"wrote {val(patch_path.name)}")

    update_blessed(base, read_rust_version(), patch_path.name)
    info("make-patch done!")


def clone_rust_repo() -> None:
    """Nuke whatever is at the repo path and do a fresh, full clone."""
    if RUST_DIR.exists():
        info(f"{val(RUST_DIR)} is not a git repo, removing it")
        shutil.rmtree(RUST_DIR)
    info(f"cloning {val(RUST_REPO)}")
    RUST_DIR.parent.mkdir(parents=True, exist_ok=True)
    run("git", "clone", "--progress", RUST_REPO, str(RUST_DIR))


def make_patch_branch(patch: "Patch") -> None:
    """Remake the patch branch by applying the patch on top of its base revision."""
    info(f"remaking the {val(PATCH_BRANCH)} branch with {val(patch.path.name)}")
    if not commit_exists(patch.revision):
        info(f"revision {val(patch.revision)} is missing, fetching it")
        git("fetch", "origin", patch.revision, "--progress")
        if not commit_exists(patch.revision):
            raise Bail(f"cannot find revision {patch.revision} that the patch is based on")
    git("checkout", "-B", PATCH_BRANCH, patch.revision)
    git("apply", "--index", "--whitespace=nowarn", str(patch.path.resolve()))
    git("commit", "--quiet", "-m", f"[megaton] apply patch {patch.date} on {patch.revision}")


class Patch(NamedTuple):
    path: Path
    date: str  # yyyy-mm-dd
    revision: str  # the base revision, REVISION_LEN chars


def find_latest_patch() -> Patch | None:
    """Find the patch with the latest date, or None if there is no patch at all."""
    patches = []
    for path in sorted(SCRIPTS_DIR.iterdir()):
        match = PATCH_RE.match(path.name)
        if match is not None:
            patches.append(Patch(path, match[1], match[2]))
    if not patches:
        return None
    latest = max(patch.date for patch in patches)
    same_date = [patch for patch in patches if patch.date == latest]
    if len(same_date) > 1:
        names = ", ".join(patch.path.name for patch in same_date)
        raise Bail(f"multiple patches of different revisions on {latest}: {names}")
    return same_date[0]


def update_change_id() -> None:
    """Bake the bootstrap change-id into our bootstrap config, as its own commit."""
    change_id = read_change_id()
    toml_path = RUST_DIR / BOOTSTRAP_TOML
    if not toml_path.is_file():
        raise Bail(f"'{BOOTSTRAP_TOML}' is missing from the rust repo, the patch should provide it")
    source = toml_path.read_text()
    updated, count = re.subn(r"(?m)^(change-id\s*=\s*).*$", rf"\g<1>{change_id}", source)
    if count != 1:
        raise Bail(f"expected exactly 1 change-id in '{BOOTSTRAP_TOML}', found {count}")
    if updated == source:
        info(f"change-id {val(change_id)} is already up to date")
        return
    toml_path.write_text(updated)
    info(f"updating change-id to {val(change_id)} in {val(BOOTSTRAP_TOML)}")
    git("add", "--", BOOTSTRAP_TOML)
    git("commit", "--quiet", "-m", f"[megaton] update change-id to {change_id}")


def read_change_id() -> str:
    """Read the latest bootstrap change-id, same as get_change_id in rust_toolchain.rs."""
    tracker_path = RUST_DIR / CHANGE_TRACKER_RS
    try:
        source = tracker_path.read_text()
    except OSError as e:
        raise Bail(f"failed to read '{tracker_path}': {e}") from e
    for line in reversed(source.splitlines()):
        line = line.strip()
        if line.startswith("change_id: "):
            return line.removeprefix("change_id: ").strip(",")
    raise Bail(f"cannot find change-id from '{CHANGE_TRACKER_RS}'")


def read_rust_version() -> str:
    """Read the compiler version from the checked-out source, like `1.101.0-dev`."""
    version_path = RUST_DIR / "src" / "version"
    try:
        version = version_path.read_text().strip()
    except OSError as e:
        raise Bail(f"failed to read '{version_path}': {e}") from e
    if not version:
        raise Bail(f"'{version_path}' is empty")
    return f"{version}-dev"


def update_blessed(commit: str, version: str, patch_name: str) -> None:
    """Update the patch path and the blessed commit/version in the rust source."""
    try:
        source = TOOLCHAIN_RS.read_text()
    except OSError as e:
        raise Bail(f"failed to read '{TOOLCHAIN_RS}': {e}") from e
    source = replace_literal(source, 'BLESSED_COMMIT: &str = "', commit)
    source = replace_literal(source, 'BLESSED_VERSION: &str = "', version)
    source = replace_literal(source, 'RUST_PATCH: &[u8] = include_bytes!("../scripts/', patch_name)
    TOOLCHAIN_RS.write_text(source)
    info(f"updated BLESSED_COMMIT = {val(commit)}")
    info(f"updated BLESSED_VERSION = {val(version)}")
    info(f"updated RUST_PATCH = {val(patch_name)}")


def replace_literal(source: str, prefix: str, value: str) -> str:
    """Replace the string literal that follows the given definition prefix."""
    pattern = re.compile(rf'({re.escape(prefix)})[^"]*(")')
    source, count = pattern.subn(lambda m: m[1] + value + m[2], source)
    if count != 1:
        raise Bail(f"expected exactly 1 `{prefix}` in '{TOOLCHAIN_RS}', found {count}")
    return source


def is_git_repo(path: Path) -> bool:
    if not (path / ".git").exists():
        return False
    return git("rev-parse", "--git-dir", check=False, quiet=True) == 0


def is_dirty() -> bool:
    # untracked files (build artifacts, bootstrap.toml, ...) are not our business
    return bool(git_out("status", "--porcelain", "--untracked-files=no"))


def commit_exists(revision: str) -> bool:
    return git("rev-parse", "--verify", f"{revision}^{{commit}}", check=False, quiet=True) == 0


def rev_parse(revision: str) -> str:
    out = git_out("rev-parse", "--verify", f"{revision}^{{commit}}")
    if not out:
        raise Bail(f"failed to resolve revision '{revision}'")
    return out


def git(*args: str, check: bool = True, quiet: bool = False) -> int:
    """Run git in the rust repo, inheriting stdout/stderr unless quiet."""
    return run("git", "-C", str(RUST_DIR), *args, check=check, quiet=quiet)


def git_out(*args: str) -> str:
    return git_bytes(*args).decode(errors="replace").strip()


def git_bytes(*args: str) -> bytes:
    command = ["git", "-C", str(RUST_DIR), *args]
    result = subprocess.run(command, stdout=subprocess.PIPE)
    if result.returncode != 0:
        raise Bail(f"command failed: {' '.join(command)}")
    return result.stdout


def run(*command: str, check: bool = True, quiet: bool = False) -> int:
    output = subprocess.DEVNULL if quiet else None
    result = subprocess.run(command, stdout=output, stderr=output)
    if check and result.returncode != 0:
        raise Bail(f"command failed: {' '.join(command)}")
    return result.returncode


class Bail(Exception):
    """An error that aborts the script with a message."""


def info(message: object) -> None:
    print(f"\x1b[96m==> {message}\x1b[0m", flush=True)


def val(value: object) -> str:
    """Highlight a value inside an informational message, back to cyan after."""
    return f"\x1b[33m{value}\x1b[96m"


def error(message: object) -> None:
    print(f"\x1b[31m==> error!! {message}\x1b[0m", file=sys.stderr, flush=True)


if __name__ == "__main__":
    sys.exit(main())
