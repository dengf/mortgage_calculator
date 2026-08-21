#!/usr/bin/env python3
"""Generate every platform's app icon from one piece of vector-ish artwork.

Run from the repository root:

    python3 assets/icon/generate.py

Requires only Pillow (plus macOS `iconutil` for the .icns, which is skipped
with a warning elsewhere). Everything it writes is committed, so this only
needs re-running when the artwork itself changes.

The mark is a house silhouette with a mortgage's real remaining-balance curve
carved through it. The curve is not decorative: it's B(t) for a level-payment
loan, which is why it stays high through the first half of the term and only
breaks late. See `amortization_curve`.
"""

import json
import os
import shutil
import subprocess
import sys

from PIL import Image, ImageDraw

SS = 4  # supersample factor; artwork is drawn at SS x and LANCZOS-downsampled

# Brand palette, taken from the existing Slint and React UIs so the icon
# matches what the user sees the moment the app opens.
NAVY_DEEP = (11, 17, 25)     # #0b1119  slint window background
NAVY_PANEL = (22, 30, 43)    # #161e2b  web panel background
BLUE = (79, 140, 255)        # #4f8cff  slint accent
GREEN = (126, 231, 135)      # #7ee787  shared "positive" green

# House silhouette in a normalized 0..1 square, y pointing down.
HOUSE = [
    (0.500, 0.115),
    (0.905, 0.442),
    (0.905, 0.885),
    (0.095, 0.885),
    (0.095, 0.442),
]


def amortization_curve(samples=200, annual_rate=0.06, years=30):
    """Remaining balance B(t)/P for a level-payment loan, sampled over the term.

    Convex and decreasing: at the halfway point roughly 71% of the principal
    is still outstanding, which gives the curve its characteristic late break.
    """
    r = annual_rate / 12.0
    n = years * 12
    denom = (1.0 + r) ** n - 1.0
    return [
        (i / samples, 1.0 - (((1.0 + r) ** (n * i / samples)) - 1.0) / denom)
        for i in range(samples + 1)
    ]


def lerp(a, b, t):
    return a + (b - a) * t


def mix(c1, c2, t):
    return tuple(int(round(lerp(c1[i], c2[i], t))) for i in range(3))


def to_px(pt, size, inset):
    """Normalized point -> pixel, honoring a safe-zone inset."""
    span = size * (1.0 - 2 * inset)
    off = size * inset
    return (off + pt[0] * span, off + pt[1] * span)


def stroke(draw, pts, size, inset, width, color):
    """Polyline with round caps and joints.

    Deliberately not `draw.line(..., joint="curve")`: on a densely sampled
    path like this one, Pillow's joint rendering leaves a comb of jagged
    spurs along the outer edge. Stamping a disc at every vertex instead
    gives a clean round-joined stroke, and the samples are close enough
    together that the discs overlap heavily.
    """
    px = [to_px(p, size, inset) for p in pts]
    w = max(1, int(width * size))
    r = w / 2.0
    draw.line(px, fill=color, width=w)
    for p in px:
        draw.ellipse([p[0] - r, p[1] - r, p[0] + r, p[1] + r], fill=color)


def vertical_gradient(size, top, bottom):
    """Background wash, just enough to keep the ground from reading flat."""
    img = Image.new("RGB", (1, size), top)
    d = ImageDraw.Draw(img)
    for y in range(size):
        d.point((0, y), fill=mix(top, bottom, y / max(1, size - 1)))
    return img.resize((size, size), Image.BILINEAR)


def draw_mark(img, size, inset):
    """The house-with-carved-curve mark, composited onto `img`."""
    house_px = [to_px(p, size, inset) for p in HOUSE]

    body = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ImageDraw.Draw(body).polygon(house_px, fill=BLUE)

    # The curve is carved out of the body rather than drawn on top: a wide
    # ground-coloured stroke first, then the green one inside it. Clipped to
    # the silhouette so nothing spills past the roofline.
    cut = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    cd = ImageDraw.Draw(cut)
    shaped = [(lerp(0.045, 0.955, p[0]), lerp(0.930, 0.400, p[1]))
              for p in amortization_curve()]
    stroke(cd, shaped, size, inset, 0.155, NAVY_DEEP)
    stroke(cd, [(p[0], p[1] + 0.004) for p in shaped], size, inset, 0.072, GREEN)

    clip = Image.new("L", (size, size), 0)
    ImageDraw.Draw(clip).polygon(house_px, fill=255)
    cut.putalpha(Image.composite(cut.getchannel("A"),
                                 Image.new("L", (size, size), 0), clip))
    body.alpha_composite(cut)
    img.alpha_composite(body)


def render(size, inset=0.0, radius=None, opaque=False):
    """Render the icon.

    inset   safe-zone padding for launchers that mask the icon (Android).
    radius  corner radius as a fraction of size; None leaves it square, which
            is what iOS and Android want since they apply their own mask.
    opaque  drop the alpha channel -- required for App Store submission.
    """
    s = size * SS
    base = vertical_gradient(s, NAVY_PANEL, NAVY_DEEP).convert("RGBA")

    if radius is not None:
        mask = Image.new("L", (s, s), 0)
        ImageDraw.Draw(mask).rounded_rectangle(
            [0, 0, s - 1, s - 1], radius=int(radius * s), fill=255)
        base.putalpha(mask)

    art = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    draw_mark(art, s, inset)
    base.alpha_composite(art)

    out = base.resize((size, size), Image.LANCZOS)
    return out.convert("RGB") if opaque else out


def write(img, *parts):
    path = os.path.join(*parts)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    img.save(path)
    print(f"  {path}  ({img.size[0]}x{img.size[1]})")
    return path


# --- iOS -------------------------------------------------------------------
# Xcode 14+ accepts a single 1024x1024 "universal" entry and derives the rest.
# The App Store rejects icons with an alpha channel, hence opaque=True.

APPICON_CONTENTS = {
    "images": [{
        "filename": "AppIcon-1024.png",
        "idiom": "universal",
        "platform": "ios",
        "scale": "1x",
        "size": "1024x1024",
    }],
    "info": {"author": "xcode", "version": 1},
}

XCASSETS_CONTENTS = {"info": {"author": "xcode", "version": 1}}


def build_ios(root):
    print("iOS")
    cat = os.path.join(root, "crates/mortgage-ui-slint/ios/Assets.xcassets")
    appicon = os.path.join(cat, "AppIcon.appiconset")
    os.makedirs(appicon, exist_ok=True)
    write(render(1024, opaque=True), appicon, "AppIcon-1024.png")
    for path, data in ((os.path.join(cat, "Contents.json"), XCASSETS_CONTENTS),
                       (os.path.join(appicon, "Contents.json"), APPICON_CONTENTS)):
        with open(path, "w") as fh:
            json.dump(data, fh, indent=2)
            fh.write("\n")
        print(f"  {path}")


# --- Android ---------------------------------------------------------------
# xbuild scales this single PNG into legacy mipmap densities; it does not
# generate adaptive (foreground/background) layers, so launchers apply their
# own mask to the square. The inset keeps the house corners inside the
# inscribed circle -- without it a round mask clips the eaves.

def build_android(root):
    print("Android")
    write(render(512, inset=0.10),
          root, "crates/mortgage-ui-slint/assets/android-icon.png")


# --- Desktop ---------------------------------------------------------------

ICNS_VARIANTS = [
    ("icon_16x16.png", 16), ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32), ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128), ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256), ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512), ("icon_512x512@2x.png", 1024),
]


def build_desktop(root, here):
    print("Desktop")
    # Slint's Window.icon, used for the title bar and taskbar entry.
    write(render(256, radius=0.22),
          root, "crates/mortgage-ui-slint/ui/app-icon.png")

    # Windows .ico, multi-resolution in one file.
    ico = os.path.join(here, "icon-windows.ico")
    render(256, radius=0.22).save(
        ico, sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    print(f"  {ico}")

    # macOS .icns. Apple's own icons sit inset inside the canvas with a
    # squircle, unlike iOS where the system masks a full-bleed square.
    if not shutil.which("iconutil"):
        print("  (skipping .icns -- iconutil not found, macOS only)")
        return
    iconset = os.path.join(here, "icon.iconset")
    shutil.rmtree(iconset, ignore_errors=True)
    os.makedirs(iconset)
    for name, size in ICNS_VARIANTS:
        pad = int(size * 0.06)
        canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        canvas.alpha_composite(render(size - 2 * pad, radius=0.22), (pad, pad))
        canvas.save(os.path.join(iconset, name))
    icns = os.path.join(here, "icon-macos.icns")
    subprocess.run(["iconutil", "-c", "icns", iconset, "-o", icns], check=True)
    shutil.rmtree(iconset)
    print(f"  {icns}")


# --- Web -------------------------------------------------------------------
# Two separate web front ends ship from this repo: the React app in www/ and
# the Slint wasm canvas in crates/mortgage-ui-slint/.

def build_web(root):
    print("Web")
    static = os.path.join(root, "www/static")
    os.makedirs(static, exist_ok=True)

    render(64, radius=0.22).save(
        os.path.join(static, "favicon.ico"),
        sizes=[(16, 16), (32, 32), (48, 48)])
    print(f"  {os.path.join(static, 'favicon.ico')}")

    write(render(32, radius=0.22), static, "favicon-32.png")
    # iOS masks the home-screen icon itself, so this one is full-bleed square.
    write(render(180, opaque=True), static, "apple-touch-icon.png")
    write(render(192, radius=0.22), static, "icon-192.png")
    write(render(512, radius=0.22), static, "icon-512.png")
    # "maskable" promises the platform it may crop to any shape it likes.
    write(render(512, inset=0.10), static, "icon-maskable-512.png")

    slint_web = os.path.join(root, "crates/mortgage-ui-slint")
    render(64, radius=0.22).save(
        os.path.join(slint_web, "favicon.ico"),
        sizes=[(16, 16), (32, 32), (48, 48)])
    print(f"  {os.path.join(slint_web, 'favicon.ico')}")


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    root = os.path.abspath(os.path.join(here, "..", ".."))
    if not os.path.isdir(os.path.join(root, "crates")):
        sys.exit(f"expected a crates/ directory under {root}")

    print("Master")
    write(render(1024, opaque=True), here, "icon-master.png")
    build_ios(root)
    build_android(root)
    build_desktop(root, here)
    build_web(root)
    print("\ndone")


if __name__ == "__main__":
    main()
