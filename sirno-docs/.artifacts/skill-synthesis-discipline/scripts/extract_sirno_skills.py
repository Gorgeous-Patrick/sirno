#!/usr/bin/env python3
"""Extract packaged Sirno skills from exact lake-owned skill artifacts."""

from __future__ import annotations

import argparse
import difflib
import re
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path


RENDER_TARGET_PATTERN = re.compile(
    r"^It renders to `(?P<target>\.agents/skills/sirno-[^`]+/SKILL\.md)`\.$",
    re.MULTILINE,
)
GENERATED_FOOTER_PATTERN = re.compile(
    r"\n---\n\n> \*\*Sirno generated links begin\. Do not edit this section\.\*\*.*\Z",
    re.DOTALL,
)
SKILL_ARTIFACT_PATH = Path("SKILL.md")


@dataclass(frozen=True)
class Entry:
    id: str
    path: Path
    metadata: dict[str, object]
    body: str


@dataclass(frozen=True)
class SkillSource:
    entry: Entry
    source: Path
    target: Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Copy exact `SKILL.md` entry artifacts into `.agents/skills/sirno-*` packages."
        ),
    )
    parser.add_argument(
        "-C",
        "--config",
        default="Sirno.toml",
        type=Path,
        help="Sirno project config path.",
    )
    parser.add_argument(
        "-L",
        "--lake-path",
        type=Path,
        help="Public Sirno Lake path. Defaults to [lake].path from the config.",
    )
    parser.add_argument(
        "--write",
        action="store_true",
        help="Write exact skill artifacts to their package paths.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Fail if any package differs from its exact skill artifact.",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List discovered skill artifact sources and package targets.",
    )
    args = parser.parse_args()

    if args.write and args.check:
        parser.error("--write and --check are mutually exclusive")

    config_path = args.config.resolve()
    root = config_path.parent
    lake_path = args.lake_path or configured_lake_path(config_path)
    if not lake_path.is_absolute():
        lake_path = root / lake_path

    entries = read_entries(lake_path)
    sources = discover_skill_sources(lake_path, entries)

    if args.list:
        for source in sources:
            print(f"{source.entry.id}\t{source.source}\t{source.target}")
        return 0

    if args.check:
        return check_sources(root, sources)
    if args.write:
        return write_sources(root, sources)

    for source in sources:
        print(f"--- {source.target} ({source.entry.id}) ---")
        content = source.source.read_bytes()
        sys.stdout.buffer.write(content)
        if not content.endswith(b"\n"):
            print()
    return 0


def configured_lake_path(config_path: Path) -> Path:
    try:
        config = tomllib.loads(config_path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise SystemExit(f"config not found: {config_path}") from None
    except tomllib.TOMLDecodeError as error:
        raise SystemExit(f"config parse error in {config_path}: {error}") from None

    path = config.get("lake", {}).get("path")
    if not isinstance(path, str) or not path:
        raise SystemExit(f"{config_path} does not define [lake].path")
    return Path(path)


def read_entries(lake_path: Path) -> dict[str, Entry]:
    if not lake_path.is_dir():
        raise SystemExit(f"lake path is not a directory: {lake_path}")

    entries = {}
    for path in sorted(lake_path.glob("*.md")):
        entry = read_entry(path)
        if entry.id in entries:
            raise SystemExit(f"duplicate entry id: {entry.id}")
        entries[entry.id] = entry
    return entries


def read_entry(path: Path) -> Entry:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        raise SystemExit(f"entry has no frontmatter: {path}")
    try:
        _, metadata_text, body = text.split("---\n", 2)
    except ValueError:
        raise SystemExit(f"entry has malformed frontmatter: {path}") from None

    metadata = parse_simple_yaml(metadata_text, path)
    return Entry(path.stem, path, metadata, strip_generated_footer(body).strip())


def parse_simple_yaml(text: str, path: Path) -> dict[str, object]:
    metadata: dict[str, object] = {}
    current_list: str | None = None
    for raw_line in text.splitlines():
        line = raw_line.rstrip()
        if not line:
            continue
        if line.startswith("  - "):
            if current_list is None:
                raise SystemExit(f"list item without key in {path}: {line}")
            metadata.setdefault(current_list, []).append(line[4:].strip())
            continue
        current_list = None
        if ":" not in line:
            raise SystemExit(f"unsupported frontmatter line in {path}: {line}")
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip()
        if value:
            metadata[key] = value
        else:
            metadata[key] = []
            current_list = key
    return metadata


def strip_generated_footer(body: str) -> str:
    return GENERATED_FOOTER_PATTERN.sub("", body)


def discover_skill_sources(lake_path: Path, entries: dict[str, Entry]) -> list[SkillSource]:
    sources = []
    for entry in entries.values():
        belongs = list_value(entry.metadata.get("belongs"))
        if "agent-skills" not in belongs:
            continue
        if not entry.id.endswith("-discipline"):
            continue

        target = render_target(entry)
        if target is None:
            continue

        source = lake_path / ".artifacts" / entry.id / SKILL_ARTIFACT_PATH
        if not source.is_file():
            raise SystemExit(f"{entry.id} is missing skill artifact: {source}")
        sources.append(SkillSource(entry, source, Path(target)))

    sources.sort(key=lambda source: source.target.as_posix())
    return sources


def list_value(value: object) -> list[str]:
    return value if isinstance(value, list) else []


def render_target(entry: Entry) -> str | None:
    match = RENDER_TARGET_PATTERN.search(entry.body)
    return match.group("target") if match else None


def check_sources(root: Path, sources: list[SkillSource]) -> int:
    failed = False
    for source in sources:
        target = root / source.target
        expected = source.source.read_bytes()
        current = target.read_bytes() if target.exists() else b""
        if current == expected:
            continue
        failed = True
        diff = difflib.unified_diff(
            current.decode("utf-8", errors="replace").splitlines(keepends=True),
            expected.decode("utf-8", errors="replace").splitlines(keepends=True),
            fromfile=str(source.target),
            tofile=f"{source.source} ({source.entry.id})",
        )
        sys.stdout.writelines(diff)
    return 1 if failed else 0


def write_sources(root: Path, sources: list[SkillSource]) -> int:
    for source in sources:
        target = root / source.target
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(source.source.read_bytes())
        print(f"wrote {source.target} from {source.source}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
