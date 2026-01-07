#!/usr/bin/env python3
"""Generate the Unfold app icon - minimal, abstract, geometric with curly braces."""

from PIL import Image, ImageDraw
import math

def draw_rounded_rect(draw, bbox, radius, fill=None):
    """Draw a rounded rectangle."""
    x1, y1, x2, y2 = bbox

    if fill:
        draw.rectangle([x1 + radius, y1, x2 - radius, y2], fill=fill)
        draw.rectangle([x1, y1 + radius, x2, y2 - radius], fill=fill)
        draw.pieslice([x1, y1, x1 + 2*radius, y1 + 2*radius], 180, 270, fill=fill)
        draw.pieslice([x2 - 2*radius, y1, x2, y1 + 2*radius], 270, 360, fill=fill)
        draw.pieslice([x1, y2 - 2*radius, x1 + 2*radius, y2], 90, 180, fill=fill)
        draw.pieslice([x2 - 2*radius, y2 - 2*radius, x2, y2], 0, 90, fill=fill)

def cubic_bezier(p0, p1, p2, p3, t):
    """Calculate point on cubic bezier curve at parameter t."""
    u = 1 - t
    return (
        u*u*u*p0[0] + 3*u*u*t*p1[0] + 3*u*t*t*p2[0] + t*t*t*p3[0],
        u*u*u*p0[1] + 3*u*u*t*p1[1] + 3*u*t*t*p2[1] + t*t*t*p3[1]
    )

def draw_curly_brace(draw, cx, cy, height, line_width, color, facing='left'):
    """Draw a proper curly brace { or } using bezier curves.

    A curly brace has:
    - Vertical stems at top and bottom that curve toward center
    - A sharp pointed tip in the middle
    """
    h = height / 2
    # How far the vertical stems are from center (less = straighter stems)
    stem_offset = height * 0.01
    # How far the middle tip extends (more = more curly)
    tip_extend = height * 0.22

    # Flip direction based on facing
    # 'left' means { (tip points left), 'right' means } (tip points right)
    d = -1 if facing == 'left' else 1

    # For { brace:
    #   - Top/bottom stems are on the RIGHT of center (positive x offset)
    #   - Tip points LEFT (negative x offset)
    # For } brace:
    #   - Top/bottom stems are on the LEFT of center (negative x offset)
    #   - Tip points RIGHT (positive x offset)

    # Top half: from top stem down to middle tip
    # Start at top (stem side), curve down to tip
    top_start = (cx - d * stem_offset, cy - h)
    top_ctrl1 = (cx - d * stem_offset, cy - h * 0.35)  # Keep vertical near top
    top_ctrl2 = (cx + d * tip_extend, cy - h * 0.15)   # Curve toward tip
    top_end = (cx + d * tip_extend, cy)                # Tip point

    # Bottom half: from tip down to bottom stem
    bot_start = (cx + d * tip_extend, cy)              # Tip point
    bot_ctrl1 = (cx + d * tip_extend, cy + h * 0.15)   # Curve away from tip
    bot_ctrl2 = (cx - d * stem_offset, cy + h * 0.35)  # Keep vertical near bottom
    bot_end = (cx - d * stem_offset, cy + h)           # Bottom stem

    # Draw the curves by plotting many small circles along the path
    steps = 200
    for i in range(steps + 1):
        t = i / steps
        r = line_width / 2

        # Top half
        x, y = cubic_bezier(top_start, top_ctrl1, top_ctrl2, top_end, t)
        draw.ellipse([x - r, y - r, x + r, y + r], fill=color)

        # Bottom half
        x, y = cubic_bezier(bot_start, bot_ctrl1, bot_ctrl2, bot_end, t)
        draw.ellipse([x - r, y - r, x + r, y + r], fill=color)

def create_unfold_icon(size=1024):
    """Create the Unfold app icon with curly braces."""

    # Gradient colors
    color1 = (64, 192, 180)   # Teal
    color2 = (80, 140, 200)   # Blue

    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))

    # Create smooth diagonal gradient
    for y in range(size):
        for x in range(size):
            ratio = (x + y) / (2 * size)
            r = int(color1[0] + (color2[0] - color1[0]) * ratio)
            g = int(color1[1] + (color2[1] - color1[1]) * ratio)
            b = int(color1[2] + (color2[2] - color1[2]) * ratio)
            img.putpixel((x, y), (r, g, b, 255))

    # Apply rounded corners
    corner_radius = int(size * 0.22)
    mask = Image.new('L', (size, size), 0)
    mask_draw = ImageDraw.Draw(mask)
    draw_rounded_rect(mask_draw, [0, 0, size-1, size-1], corner_radius, fill=255)

    result = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    result.paste(img, mask=mask)

    draw = ImageDraw.Draw(result)

    cx, cy = size // 2, size // 2

    # Brace parameters
    height = size * 0.55
    line_width = size * 0.045

    # Colors
    white_solid = (255, 255, 255, 230)
    white_faded = (255, 255, 255, 80)

    # Inner braces (solid) - the main { }
    # Opening brace { on left (faces left, tip points left)
    # Closing brace } on right (faces right, tip points right)
    # Moved outward slightly to create gap from center dots
    inner_offset = size * 0.18
    draw_curly_brace(draw, cx - inner_offset, cy, height, line_width, white_solid, 'left')
    draw_curly_brace(draw, cx + inner_offset, cy, height, line_width, white_solid, 'right')

    # Three dots at the bottom (like { ... } with content indicator below)
    dot_radius = size * 0.025
    dot_spacing = size * 0.07
    dot_y = cy + height * 0.48  # Position below the braces with slight gap
    for i in range(-1, 2):  # -1, 0, 1
        dot_x = cx + i * dot_spacing
        draw.ellipse([dot_x - dot_radius, dot_y - dot_radius,
                     dot_x + dot_radius, dot_y + dot_radius], fill=white_solid)

    # Outer braces (faded) - suggesting "unfolding" outward
    # Same pattern: { on left, } on right
    outer_offset = size * 0.28
    draw_curly_brace(draw, cx - outer_offset, cy, height, line_width, white_faded, 'left')
    draw_curly_brace(draw, cx + outer_offset, cy, height, line_width, white_faded, 'right')

    return result

if __name__ == '__main__':
    import os

    os.makedirs('/Users/maumercado/code/codigo-projects/unfold/assets', exist_ok=True)

    sizes = [1024, 512, 256, 128, 64, 32, 16]

    icon = create_unfold_icon(1024)
    icon.save('/Users/maumercado/code/codigo-projects/unfold/assets/icon-1024.png', 'PNG')
    print('Created icon-1024.png')

    for s in sizes[1:]:
        resized = icon.resize((s, s), Image.Resampling.LANCZOS)
        resized.save(f'/Users/maumercado/code/codigo-projects/unfold/assets/icon-{s}.png', 'PNG')
        print(f'Created icon-{s}.png')

    print('\nAll icons generated successfully!')
