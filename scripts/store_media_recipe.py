"""Deterministically derive store delivery images from approved project masters."""

from pathlib import Path
from PIL import Image, ImageOps

ROOT = Path(__file__).resolve().parents[1]
MEDIA = ROOT / "docs" / "release" / "store_media"
MASTERS = MEDIA / "masters"
DELIVERY = MEDIA / "delivery"


def cover(image: Image.Image, size: tuple[int, int], center=(0.5, 0.5)) -> Image.Image:
    source_ratio = image.width / image.height
    target_ratio = size[0] / size[1]
    if source_ratio > target_ratio:
        width = round(image.height * target_ratio)
        left = round((image.width - width) * center[0])
        image = image.crop((left, 0, left + width, image.height))
    else:
        height = round(image.width / target_ratio)
        top = round((image.height - height) * center[1])
        image = image.crop((0, top, image.width, top + height))
    return image.resize(size, Image.Resampling.LANCZOS)


def branded(size: tuple[int, int], logo_fraction: float, center=(0.5, 0.5)) -> Image.Image:
    canvas = cover(hero, size, center).convert("RGBA")
    logo_width = round(size[0] * logo_fraction)
    logo_height = round(logo.height * logo_width / logo.width)
    max_height = round(size[1] * 0.45)
    if logo_height > max_height:
        logo_height = max_height
        logo_width = round(logo.width * logo_height / logo.height)
    mark = logo.resize((logo_width, logo_height), Image.Resampling.LANCZOS)
    x = (size[0] - logo_width) // 2
    y = max(round(size[1] * 0.05), (size[1] - logo_height) // 8)
    canvas.alpha_composite(mark, (x, y))
    return canvas.convert("RGB")


DELIVERY.mkdir(parents=True, exist_ok=True)
hero = Image.open(MASTERS / "stellar_legacy_key_art_no_text.png").convert("RGB")
logo = Image.open(MASTERS / "stellar_legacy_logo_transparent.png").convert("RGBA")
icon = Image.open(ROOT / "assets" / "packaging" / "stellar_legacy_icon_master.png").convert("RGB")

assets = {
    "steam_header_920x430.png": branded((920, 430), 0.62, (0.55, 0.55)),
    "steam_small_462x174.png": branded((462, 174), 0.58, (0.55, 0.55)),
    "steam_main_1232x706.png": branded((1232, 706), 0.60, (0.55, 0.52)),
    "steam_vertical_748x896.png": branded((748, 896), 0.78, (0.72, 0.52)),
    "steam_library_capsule_600x900.png": branded((600, 900), 0.82, (0.70, 0.52)),
    "steam_library_header_920x430.png": branded((920, 430), 0.62, (0.55, 0.55)),
    "steam_library_hero_3840x1240.png": cover(hero, (3840, 1240), (0.52, 0.58)),
    "steam_shortcut_icon_256x256.png": cover(icon, (256, 256)),
    "steam_app_icon_184x184.png": cover(icon, (184, 184)),
    "itch_cover_630x500.png": branded((630, 500), 0.72, (0.58, 0.55)),
}
for name, image in assets.items():
    image.save(DELIVERY / name, optimize=True)

logo.thumbnail((1280, 720), Image.Resampling.LANCZOS)
logo.save(DELIVERY / "steam_library_logo_transparent.png", optimize=True)

screens = ["gameplay", "event", "contracts", "ship", "debrief", "dashboard_risk"]
for index, scene in enumerate(screens, 1):
    source = ROOT / "docs" / "verification" / f"ui_{scene}.png"
    if source.exists():
        frame = Image.open(source).convert("RGB")
        if frame.width < 1920 or frame.height < 1000:
            raise SystemExit(f"Store screenshot is below the storefront capture floor: {source}")
        # Windows reports the client framebuffer below the requested outer-window
        # height (1061px on a 1080p desktop). Letterbox, never stretch or invent UI.
        frame = ImageOps.pad(frame, (1920, 1080), color="black", method=Image.Resampling.LANCZOS)
        frame.save(DELIVERY / f"screenshot_{index:02d}_{scene}.png", optimize=True)

print(f"Generated store media in {DELIVERY}")
