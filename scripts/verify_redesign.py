"""Capture the published release through the toolkit harness and verify PNG dimensions.

Run publish.ps1 first, then: python scripts/verify_redesign.py
The matrix is explicit, reviewable, and independent of Git or user save contents.
"""
import json
import os
import re
import subprocess
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
MATRIX = json.loads((ROOT / "docs/ui_redesign_capture_manifest.json").read_text(encoding="utf-8"))
SIZES = {"baseline": (1280, 720), "portrait": (390, 844), "compact": (1024, 768), "desktop": (1920, 1080)}
OUT = ROOT / "docs/verification"
TEMP = ROOT / "target-demo"
TEMP.mkdir(exist_ok=True)
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=ROOT))
exe = Path(metadata["target_directory"]) / "release/stellar_legacy.exe"
if not exe.is_file():
    raise SystemExit("Run publish.ps1 first: the release executable is missing.")

results = []
audits = []
for viewport, scenes in MATRIX.items():
    width, height = SIZES[viewport]
    manifest = TEMP / f"capture-{viewport}.tsv"
    rows = []
    for scene in scenes:
        safe = re.sub(r"[^A-Za-z0-9._+-]", "_", scene)
        suffix = "" if viewport == "baseline" or scene.endswith("_narrow") else f"_{viewport}"
        path = OUT / f"ui_{safe}{suffix}.png"
        rows.append((scene, path))
    manifest.write_text("\n".join(f"{scene}\t{path}" for scene, path in rows), encoding="utf-8")
    env = dict(os.environ, STELLAR_LEGACY_CAPTURE_MANIFEST=str(manifest), STELLAR_LEGACY_CAPTURE_FRAMES="3",
               STELLAR_LEGACY_CAPTURE_FULLSCREEN="1" if viewport == "desktop" else "0",
               STELLAR_LEGACY_HEADLESS="1", STELLAR_LEGACY_WINDOW_WIDTH=str(width), STELLAR_LEGACY_WINDOW_HEIGHT=str(height))
    run = subprocess.run([str(exe)], cwd=ROOT, env=env, text=True, encoding="utf-8", capture_output=True, timeout=180)
    log = run.stdout + run.stderr
    (TEMP / f"capture-{viewport}.log").write_text(log, encoding="utf-8")
    if run.returncode:
        raise RuntimeError(log)
    assert "AMBIGUOUS:" not in log, f"Overlapping targets in {viewport}: see capture log"
    audits.extend(f"{viewport}: {line}" for line in log.splitlines() if line.startswith("touch["))
    for scene, path in rows:
        with Image.open(path) as frame:
            frame.load()
            assert frame.size == (width, height), (path, frame.size)
            assert frame.convert("RGB").getextrema() != ((0, 0), (0, 0), (0, 0)), path
        results.append({"scene": scene, "viewport": viewport, "width": width, "height": height,
                        "file": str(path.relative_to(ROOT)).replace("\\", "/")})
    print(f"{viewport}: verified {len(rows)} PNGs at {width}x{height}", flush=True)

(OUT / "redesign_dimensions.json").write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
(OUT / "redesign_touch_audit.txt").write_text("\n".join(audits) + "\n", encoding="utf-8")
print(f"{len(results)} release captures verified. Inspect the PNGs; dimensions alone do not establish readability.")
