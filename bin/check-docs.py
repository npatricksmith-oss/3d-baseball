#!/usr/bin/env python3
"""Check the working docs for drift: TASKS.md size, dead links, dead anchors,
dead backticked repo paths.

Run from any checkout:  python3 bin/check-docs.py [--all]
Exit 0 when clean; exit 1 with one `file:line: message` per problem.

Checked files: top-level *.md and docs/*.md. `--all` also walks docs/calibration/,
docs/reviews/ and docs/archive/ (dated records). Never entered: .claude, .git,
sim/target, client/Saved, client/Intermediate, client/Binaries.

Checks
- TASKS.md at most TASKS_MAX_BYTES.
- `[text](path)` / `[text](path#anchor)` / `[text](#anchor)`: target must exist
  (relative to the linking file); `#Lnn` / `#Lnn-Lmm` anchors on any target are
  line anchors and not checked; other anchors on .md targets must match a heading
  by GitHub slug rules (duplicates get -1, -2, ...). http(s):, mailto: skipped.
- Backticked spans that look like repo paths (start with sim/, client/, docs/,
  analysis/, bin/, or a top-level NAME.md; no spaces or wildcard/template
  characters) must exist, resolved from the repo root; a trailing `:line`,
  `:a-b` or `#Lnn` is allowed.
Not checked (cannot be done reliably): bare file names without a directory prefix
(`pa.rs`), paths with wildcards/placeholders, paths in fenced code blocks, and
paths under sim/target, sim/out and the ignored client folders, and WORKER-*.md briefs.
"""
import re
import sys
from pathlib import Path
from urllib.parse import unquote

TASKS_MAX_BYTES = 30 * 1024
SKIP_DIRS = {".claude", ".git"}
SKIP_PREFIXES = ("sim/target", "client/Saved", "client/Intermediate", "client/Binaries")
# Generated, git-ignored output: paths into these may legitimately be absent.
GENERATED_PREFIXES = SKIP_PREFIXES + ("sim/out",)
DATED_DIRS = ("docs/calibration", "docs/reviews", "docs/archive")
PATH_PREFIXES = ("sim/", "client/", "docs/", "analysis/", "bin/")

FENCE_RE = re.compile(r"^\s{0,3}(```|~~~)")
HEADING_RE = re.compile(r"^\s{0,3}(#{1,6})\s+(.*?)\s*#*\s*$")
LINK_RE = re.compile(r"\[(?:[^\]\\]|\\.)*\]\(\s*(<[^>]*>|[^)\s]+)(?:\s+(?:\"[^\"]*\"|'[^']*'))?\s*\)")
CODE_RE = re.compile(r"(`+)(.+?)\1")
LINE_ANCHOR_RE = re.compile(r"^L\d+(-L?\d+)?$")
PATH_SUFFIX_RE = re.compile(r"(:\d+(-\d+)?|#L\d+(-L?\d+)?)$")
BAD_PATH_CHARS = set(" \t*?<>{}[]$|\\\"'()")


def slugify(heading):
    """GitHub heading slug, without the duplicate suffix."""
    text = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", heading)  # links -> text
    text = text.lower()
    out = []
    for ch in text:
        if ch.isalnum() or ch in "-_":
            out.append(ch)
        elif ch == " ":
            out.append("-")
    return "".join(out)


def lines_outside_fences(text):
    """Yield (lineno, line) for lines not inside a fenced code block."""
    fence = None
    for n, line in enumerate(text.splitlines(), 1):
        m = FENCE_RE.match(line)
        if m:
            if fence is None:
                fence = m.group(1)
            elif m.group(1) == fence:
                fence = None
            continue
        if fence is None:
            yield n, line


def anchors_of(text):
    seen = {}
    out = set()
    for _, line in lines_outside_fences(text):
        m = HEADING_RE.match(line)
        if not m:
            continue
        slug = slugify(m.group(2))
        k = seen.get(slug, 0)
        seen[slug] = k + 1
        out.add(slug if k == 0 else f"{slug}-{k}")
    return out


def collect_files(root, include_dated):
    # WORKER-*.md are untracked job briefs, not working docs.
    files = [f for f in sorted(root.glob("*.md")) if not f.name.startswith("WORKER-")] + sorted((root / "docs").glob("*.md"))
    if include_dated:
        for d in DATED_DIRS:
            files += sorted((root / d).rglob("*.md"))
    return files


def check(root, include_dated=False):
    root = Path(root).resolve()
    problems = []
    cache = {}

    def anchors(path):
        if path not in cache:
            try:
                cache[path] = anchors_of(path.read_text(encoding="utf-8", errors="replace"))
            except OSError:
                cache[path] = set()
        return cache[path]

    tasks = root / "TASKS.md"
    if tasks.is_file() and tasks.stat().st_size > TASKS_MAX_BYTES:
        problems.append(
            f"TASKS.md:1: {tasks.stat().st_size} bytes exceeds the cap of {TASKS_MAX_BYTES}"
        )

    for f in collect_files(root, include_dated):
        rel = f.relative_to(root).as_posix()
        if any(rel.startswith(p) for p in SKIP_PREFIXES) or SKIP_DIRS & set(f.relative_to(root).parts):
            continue
        text = f.read_text(encoding="utf-8", errors="replace")
        for n, line in lines_outside_fences(text):
            # backticked repo paths
            for m in CODE_RE.finditer(line):
                span = m.group(2).strip()
                p = PATH_SUFFIX_RE.sub("", span)
                if BAD_PATH_CHARS & set(p) or "..." in p:
                    continue
                top_md = "/" not in p and p.endswith(".md")
                if not (p.startswith(PATH_PREFIXES) or top_md):
                    continue
                if p.startswith(GENERATED_PREFIXES) or p.startswith("WORKER-"):
                    continue
                if not ((root / p).exists() or (top_md and len(p) > 3 and (f.parent / p).exists())):
                    problems.append(f"{rel}:{n}: backticked path `{span}` does not exist")
            # links (ignore code spans)
            no_code = CODE_RE.sub(lambda m: " " * len(m.group(0)), line)
            for m in LINK_RE.finditer(no_code):
                target = m.group(1)
                if target.startswith("<") and target.endswith(">"):
                    target = target[1:-1]
                if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target):
                    continue  # http:, https:, mailto:, ...
                path_part, _, anchor = target.partition("#")
                anchor = unquote(anchor)
                if path_part:
                    dest = (f.parent / unquote(path_part)).resolve()
                    if not dest.exists():
                        problems.append(f"{rel}:{n}: dead link `{target}`")
                        continue
                else:
                    dest = f
                if anchor and not LINE_ANCHOR_RE.match(anchor) and dest.is_file() and dest.suffix == ".md":
                    if anchor.lower() not in anchors(dest):
                        problems.append(f"{rel}:{n}: dead anchor `{target}`")
    return problems


def main(argv):
    root = Path(__file__).resolve().parent.parent
    problems = check(root, include_dated="--all" in argv)
    for p in problems:
        print(p)
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
