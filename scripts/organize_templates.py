#!/usr/bin/env python3
"""
One-time script: reads tmp/index, copies TextFSM templates into the canonical
resource layout, and generates registry.json.
"""

import csv
import json
import os
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TMP_DIR = ROOT / "tmp"
INDEX_FILE = TMP_DIR / "index"
RESOURCES_DIR = ROOT / "crates" / "netcli_core" / "resources"
TEMPLATES_DIR = RESOURCES_DIR / "templates"
REGISTRY_FILE = RESOURCES_DIR / "registry.json"


def is_regex_platform(platform: str) -> bool:
    return any(ch in platform for ch in "(|)*[].+?")


def derive_platform_from_filename(filename: str, all_platforms: set[str]) -> str | None:
    """Find the longest known concrete platform that is a prefix of filename."""
    stem = filename.replace(".textfsm", "")
    best = None
    for p in all_platforms:
        if stem.startswith(p + "_"):
            if best is None or len(p) > len(best):
                best = p
    return best


def collect_concrete_platforms(index_path: Path) -> set[str]:
    """First pass: gather all non-regex platform names from the index."""
    platforms = set()
    with open(index_path, "r") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = [p.strip() for p in line.split(",")]
            if len(parts) < 4:
                continue
            platform = parts[2]
            if platform in ("Hostname", "Platform"):
                continue
            if not is_regex_platform(platform):
                platforms.add(platform)
    return platforms


def parse_index(index_path: Path, concrete_platforms: set[str]):
    """Yield (template_filenames_list, platform) per data line."""
    with open(index_path, "r") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = [p.strip() for p in line.split(",")]
            if len(parts) < 4:
                continue
            templates_raw = parts[0]
            platform = parts[2]
            if platform in ("Hostname", "Platform"):
                continue
            filenames = [t.strip() for t in templates_raw.split(":") if t.strip()]

            if is_regex_platform(platform):
                derived = derive_platform_from_filename(filenames[0], concrete_platforms)
                if derived:
                    platform = derived
                else:
                    stem = filenames[0].replace(".textfsm", "")
                    parts_f = stem.split("_")
                    platform = "_".join(parts_f[:2])

            yield filenames, platform


def derive_command_key(filename: str, platform: str) -> str:
    """Strip platform prefix and .textfsm suffix, normalize hyphens."""
    name = filename
    if name.endswith(".textfsm"):
        name = name[: -len(".textfsm")]
    prefix = platform + "_"
    if name.startswith(prefix):
        name = name[len(prefix):]
    return name.replace("-", "_")


def main():
    if TEMPLATES_DIR.exists():
        shutil.rmtree(TEMPLATES_DIR)
    TEMPLATES_DIR.mkdir(parents=True, exist_ok=True)

    concrete_platforms = collect_concrete_platforms(INDEX_FILE)

    registry_entries = []
    copied_files = set()

    for filenames, platform in parse_index(INDEX_FILE, concrete_platforms):
        platform_dir = TEMPLATES_DIR / platform
        platform_dir.mkdir(parents=True, exist_ok=True)

        primary = filenames[0]
        command_key = derive_command_key(primary, platform)

        for fname in filenames:
            src = TMP_DIR / fname
            dest_name = fname
            file_prefix = platform + "_"
            if dest_name.startswith(file_prefix):
                dest_name = dest_name[len(file_prefix):]
            dest = platform_dir / dest_name

            if src.exists() and str(dest) not in copied_files:
                shutil.copy2(src, dest)
                copied_files.add(str(dest))

        primary_dest = primary
        if primary_dest.startswith(platform + "_"):
            primary_dest = primary_dest[len(platform) + 1:]

        registry_entries.append(
            {
                "platform": platform,
                "commandKey": command_key,
                "template": f"templates/{platform}/{primary_dest}",
                "shape": "list",
            }
        )

    registry = {"templates": registry_entries}
    with open(REGISTRY_FILE, "w") as f:
        json.dump(registry, f, indent=2)
        f.write("\n")

    platforms = sorted(set(e["platform"] for e in registry_entries))
    print(f"Done: {len(registry_entries)} registry entries across {len(platforms)} platforms")
    print(f"Copied {len(copied_files)} template files")
    print(f"Registry written to {REGISTRY_FILE}")


if __name__ == "__main__":
    main()
