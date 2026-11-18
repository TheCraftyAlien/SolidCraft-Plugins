"""Build, package and end-to-end test the Timber plugin.

    python build.py            unit tests, wasm build, dist/timber/ and dist/timber-<version>.zip, then the end-to-end tests
    python build.py --no-e2e   the same without the end-to-end tests (no solidcraft-cli needed)

Needs: cargo with the wasm32-unknown-unknown target, wasm-tools, and for the end-to-end tests
solidcraft-cli on PATH (or in SOLIDCRAFT_CLI).
"""
import json
import os
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).parent
PKG = ROOT / "dist" / "timber"
os.environ["PATH"] += os.pathsep + str(Path.home() / ".cargo" / "bin")
CLI = os.environ.get("SOLIDCRAFT_CLI") or shutil.which("solidcraft-cli") or r"C:\Program Files\SolidCraft\solidcraft-cli.exe"


def sh(*args):
    subprocess.run(args, cwd=ROOT, check=True)


def build():
    sh("cargo", "test", "--quiet")
    sh("cargo", "build", "--release", "--target", "wasm32-unknown-unknown")
    PKG.mkdir(parents=True, exist_ok=True)
    sh("wasm-tools", "component", "new",
       "target/wasm32-unknown-unknown/release/timber.wasm", "-o", str(PKG / "timber.wasm"))
    # The app refuses a manifest over 16 KiB, so the packaged one carries no white space.
    manifest = json.loads((ROOT / "plugin.json").read_text(encoding="utf-8"))
    (PKG / "plugin.json").write_text(json.dumps(manifest, ensure_ascii=False, separators=(",", ":")),
                                     encoding="utf-8", newline="\n")
    version = manifest["version"]
    archive = ROOT / "dist" / f"timber-{version}.zip"
    with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as z:
        for name in ("plugin.json", "timber.wasm"):
            z.write(PKG / name, f"timber/{name}")
    print("built", archive)
    return archive


def cli(script):
    """Runs a command script through solidcraft-cli and returns the results."""
    path = ROOT / "dist" / "script.json"
    path.write_text(json.dumps(script), encoding="utf-8")
    out = ROOT / "dist" / "results.json"
    sh(CLI, "run", str(path), "--results", "--quiet", "--json-out", str(out))
    return json.loads(out.read_text(encoding="utf-8"))


def e2e():
    sys.path[:0] = [str(ROOT / "tests")]
    import e2e as tests
    approval = cli([{"command": "plugin.inspect", "params": {"path": str(PKG)}}])[0]["approval"]
    tests.run(cli, str(PKG), approval)


if __name__ == "__main__":
    build()
    if "--no-e2e" not in sys.argv:
        e2e()
