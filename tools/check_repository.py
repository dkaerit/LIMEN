#!/usr/bin/env python3
"""Repository policy checks that require only the Python standard library."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[1]

REQUIRED_FILES = (
    "README.md",
    "SPEC.md",
    "ARCHITECTURE.md",
    "ROADMAP.md",
    "DECISIONS.md",
    "AGENTS.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    ".github/workflows/ci.yml",
)

TEXT_SUFFIXES = {
    ".css",
    ".html",
    ".js",
    ".json",
    ".jsx",
    ".md",
    ".py",
    ".rs",
    ".sh",
    ".toml",
    ".ts",
    ".tsx",
    ".txt",
    ".yaml",
    ".yml",
}

FORBIDDEN_SUFFIXES = {
    ".appimage",
    ".bios",
    ".chd",
    ".cso",
    ".dll",
    ".dylib",
    ".exe",
    ".iso",
    ".nsp",
    ".pdb",
    ".pkg",
    ".rap",
    ".rom",
    ".rvz",
    ".sav",
    ".so",
    ".state",
    ".wad",
    ".wbfs",
    ".xci",
}

CONFLICT_MARKER = re.compile(r"^(<<<<<<<|=======|>>>>>>>)", re.MULTILINE)
MARKDOWN_LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


def repository_files() -> list[Path]:
    return sorted(
        path
        for path in ROOT.rglob("*")
        if path.is_file() and ".git" not in path.relative_to(ROOT).parts
    )


def check_required_files(errors: list[str]) -> None:
    for relative in REQUIRED_FILES:
        if not (ROOT / relative).is_file():
            errors.append(f"missing required file: {relative}")


def check_forbidden_files(files: list[Path], errors: list[str]) -> None:
    for path in files:
        if path.suffix.lower() in FORBIDDEN_SUFFIXES:
            errors.append(f"forbidden binary or protected-content extension: {path.relative_to(ROOT)}")


def check_text_files(files: list[Path], errors: list[str]) -> None:
    for path in files:
        if path.suffix.lower() not in TEXT_SUFFIXES and path.name not in {
            ".editorconfig",
            ".gitattributes",
            ".gitignore",
            "CODEOWNERS",
        }:
            continue

        relative = path.relative_to(ROOT)
        try:
            content = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            errors.append(f"text file is not valid UTF-8: {relative}")
            continue

        if content and not content.endswith("\n"):
            errors.append(f"file does not end with a newline: {relative}")
        if "\r\n" in content:
            errors.append(f"file uses CRLF instead of repository LF policy: {relative}")
        if CONFLICT_MARKER.search(content):
            errors.append(f"merge conflict marker found: {relative}")

        for number, line in enumerate(content.splitlines(), start=1):
            trailing = len(line) - len(line.rstrip(" \t"))
            if trailing:
                errors.append(f"trailing whitespace: {relative}:{number}")


def check_markdown_links(files: list[Path], errors: list[str]) -> None:
    for path in files:
        if path.suffix.lower() != ".md":
            continue

        content = path.read_text(encoding="utf-8")
        for match in MARKDOWN_LINK.finditer(content):
            raw_target = match.group(1).strip()
            if not raw_target or raw_target.startswith(("#", "http://", "https://", "mailto:")):
                continue

            target = unquote(raw_target.split("#", 1)[0])
            if not target:
                continue

            resolved = (path.parent / target).resolve()
            try:
                resolved.relative_to(ROOT)
            except ValueError:
                errors.append(f"local link leaves repository: {path.relative_to(ROOT)} -> {raw_target}")
                continue

            if not resolved.exists():
                errors.append(f"broken local link: {path.relative_to(ROOT)} -> {raw_target}")


def check_workflow(errors: list[str]) -> None:
    workflow = ROOT / ".github/workflows/ci.yml"
    if not workflow.is_file():
        return

    content = workflow.read_text(encoding="utf-8")
    if "pull_request_target:" in content:
        errors.append("CI must not use pull_request_target")
    if not re.search(r"permissions:\s*\n\s+contents:\s+read", content):
        errors.append("CI must declare read-only contents permission")


def main() -> int:
    errors: list[str] = []
    files = repository_files()

    check_required_files(errors)
    check_forbidden_files(files, errors)
    check_text_files(files, errors)
    check_markdown_links(files, errors)
    check_workflow(errors)

    if errors:
        print("Repository policy failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(f"Repository policy passed ({len(files)} files checked).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
