"""Check tracked project text and local inline Markdown links without dependencies."""

from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parents[1]
TEXT_SUFFIXES = {".md", ".rs", ".toml", ".json", ".yml", ".yaml", ".py", ".txt"}
TEXT_NAMES = {"LICENSE", ".gitignore", ".gitattributes", ".gitkeep"}
LINK = re.compile(r"\]\(([^)]+)\)")
# Split legacy spellings so the checker does not flag its own source.
OLD_NAMES = ("ins" + "landio", "insalnd" + "volt")


def main() -> int:
    tracked = subprocess.check_output(
        ["git", "ls-files", "-z"], cwd=ROOT
    ).decode("utf-8").split("\0")
    errors = []
    checked = 0
    longest = 0
    for relative in filter(None, tracked):
        path = ROOT / relative
        if path.suffix not in TEXT_SUFFIXES and path.name not in TEXT_NAMES:
            continue
        try:
            content = path.read_text(encoding="utf-8")
        except (UnicodeError, OSError) as error:
            errors.append(f"{relative}: cannot read UTF-8 text: {error}")
            continue
        checked += 1
        lines = len(content.splitlines())
        longest = max(longest, lines)
        if lines > 300:
            errors.append(f"{relative}: {lines} lines, limit is 300")
        if any(name in content.lower() for name in OLD_NAMES):
            errors.append(f"{relative}: obsolete project name")
        if path.suffix != ".md":
            continue
        # Only inline local file targets are checked; remote URLs and anchors
        # need separate verification and are intentionally outside this check.
        for match in LINK.finditer(content):
            target = match.group(1).strip()
            if target.startswith("<") and ">" in target:
                target = target[1:target.index(">")]
            else:
                target = target.split(maxsplit=1)[0] if target else ""
            parsed = urlsplit(target)
            if parsed.scheme or parsed.netloc or not parsed.path:
                continue
            base = ROOT if parsed.path.startswith("/") else path.parent
            destination = base / unquote(parsed.path.lstrip("/"))
            if not destination.exists():
                errors.append(f"{relative}: missing local target {target}")
    print(f"Checked {checked} text files; maximum {longest}/300 lines.")
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    print("Local inline file links and IslandVolt naming are valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
