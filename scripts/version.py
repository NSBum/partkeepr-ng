#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Keep the workspace, standalone backend, and lockfile release versions aligned."""
import argparse
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent


def current_version():
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    backend = tomllib.loads((ROOT / "backend/Cargo.toml").read_text())["package"]["version"]
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    locked = [p["version"] for p in packages if p["name"] == "partkeepr-api"]
    if locked != [workspace] or backend != workspace:
        raise ValueError("Version mismatch: Cargo.toml, backend/Cargo.toml, and Cargo.lock must agree.")
    if not re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", workspace):
        raise ValueError("Expected a numeric MAJOR.MINOR.PATCH version.")
    return workspace


def set_version(old, new):
    if not re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", new):
        raise ValueError("Use a numeric MAJOR.MINOR.PATCH version.")
    if tuple(map(int, new.split('.'))) <= tuple(map(int, old.split('.'))):
        raise ValueError("The new version must be greater than the current version.")
    replacements = []
    for relative in ["Cargo.toml", "backend/Cargo.toml", "Cargo.lock"]:
        path = ROOT / relative
        pattern = (r'(?m)^(name = "partkeepr-api"\nversion = ")' if relative == "Cargo.lock"
                   else r'(?m)^(version\s*=\s*")') + re.escape(old) + r'("\s*)$'
        text, count = re.subn(pattern, lambda m: m[1] + new + m[2], path.read_text())
        if count != 1:
            raise ValueError(f"Expected exactly one package version in {relative}; found {count}.")
        replacements.append((path, text))
    # Validate every replacement before writing any file.
    for path, text in replacements:
        path.write_text(text)
    print(f"{old} -> {new}; add a dated CHANGELOG.md entry before committing.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group()
    action.add_argument("--check", action="store_true", help="check version consistency and a dated changelog entry")
    action.add_argument("--set", metavar="VERSION", help="set all three version declarations")
    action.add_argument("--bump", choices=["patch", "minor", "major"], help="increment a semantic version component")
    args = parser.parse_args()
    try:
        version = current_version()
        if args.set or args.bump:
            new = args.set
            if args.bump:
                parts = list(map(int, version.split('.')))
                index = {"major": 0, "minor": 1, "patch": 2}[args.bump]
                parts[index] += 1
                parts[index + 1:] = [0] * (2 - index)
                new = '.'.join(map(str, parts))
            set_version(version, new)
        elif args.check:
            changelog = ROOT / "CHANGELOG.md"
            if not changelog.exists() or not re.search(
                r"(?m)^## \[" + re.escape(version) + r"\] - \d{4}-\d{2}-\d{2}$", changelog.read_text()
            ):
                raise ValueError(f"Missing dated CHANGELOG.md entry for {version}.")
            print(f"Version {version}: manifests, lockfile, and changelog agree.")
        else:
            print(version)
    except (ValueError, KeyError, OSError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
