#!/usr/bin/env python3
"""Generate high-quality pixel art sprite sheets for the Claude Poke cockatiel.

Character design based on reference image:
  - Round cream/white body
  - Yellow face area
  - Orange beak
  - Pink cheek blush
  - Black eyes with white highlights
  - Yellow/orange flame crest
  - Small orange feet
  - Black outline

Approach: define the base cockatiel as a pixel grid, then create animation
variants by modifying eye state, beak, wings, and adding accessories.

Each frame: 64x64 pixels, RGBA, transparent background.
Frames arranged as horizontal strip in PNG.
"""

from PIL import Image, ImageDraw
import os

FRAME_SIZE = 64

# ── Color Palette ──
TRANSPARENT = (0, 0, 0, 0)
OUTLINE     = (30, 30, 50, 255)
CREAM       = (248, 238, 218, 255)   # Body main
CREAM_LIGHT = (255, 248, 235, 255)   # Body highlight (top-left)
CREAM_DARK  = (225, 208, 178, 255)   # Body shadow (bottom-right)
YELLOW      = (255, 210, 50, 255)    # Face & crest main
YELLOW_DARK = (230, 175, 20, 255)    # Crest shadow
YELLOW_FACE = (255, 220, 70, 255)    # Face area (slightly different)
ORANGE      = (240, 130, 55, 255)    # Beak & feet
ORANGE_DARK = (210, 105, 35, 255)    # Beak shadow
PINK        = (250, 165, 165, 255)   # Cheek blush
EYE_BLACK   = (30, 30, 50, 255)      # Eye color
EYE_WHITE   = (255, 255, 255, 255)   # Eye highlight
WHITE       = (255, 255, 255, 255)
RED         = (230, 60, 60, 255)
RED_LIGHT   = (255, 80, 80, 255)
PURPLE      = (150, 90, 210, 255)
LIGHT_BLUE  = (140, 190, 230, 255)
STAR_YELLOW = (255, 220, 60, 255)


def px(draw, x, y, color):
    if 0 <= x < FRAME_SIZE and 0 <= y < FRAME_SIZE:
        draw.point((x, y), fill=color)


def draw_ellipse_outline(draw, cx, cy, rx, ry, color, thickness=1):
    """Draw an ellipse outline using parametric angle sampling."""
    import math
    points = set()
    for t in range(360):
        angle = math.radians(t)
        for dt in range(-thickness + 1, 1):
            x = int(round(cx + (rx + dt) * math.cos(angle)))
            y = int(round(cy + (ry + dt) * math.sin(angle)))
            points.add((x, y))
    for x, y in points:
        px(draw, x, y, color)


def fill_ellipse(draw, cx, cy, rx, ry, colors):
    """Fill an ellipse with conditional coloring.
    colors: dict with keys 'light', 'main', 'dark', 'outline'.
    """
    import math
    for y in range(int(cy - ry) - 1, int(cy + ry) + 2):
        for x in range(int(cx - rx) - 1, int(cx + rx) + 2):
            dx = (x - cx) / max(rx, 0.5)
            dy = (y - cy) / max(ry, 0.5)
            dist = dx * dx + dy * dy
            if dist > 1.05:
                continue
            if dist > 0.92:
                px(draw, x, y, colors.get('outline', OUTLINE))
            else:
                shade = dx + dy
                if shade < -0.3:
                    px(draw, x, y, colors.get('light', CREAM_LIGHT))
                elif shade > 0.3:
                    px(draw, x, y, colors.get('dark', CREAM_DARK))
                else:
                    px(draw, x, y, colors.get('main', CREAM))


def draw_base_bird(draw, ox=0, oy=0, eye_state="open", look="center",
                   beak_open=False, wing_fold="normal"):
    """Draw the base cockatiel matching the reference image.

    eye_state: "open", "closed", "half", "wide", "x_eyes"
    look: "center", "left", "right", "up"
    wing_fold: "normal", "raised", "spread", "down"
    """
    bird_cx = 32 + ox
    bird_cy = 36 + oy

    # ── 1. Body (large round shape) ──
    body_cx, body_cy = bird_cx, bird_cy + 2
    fill_ellipse(draw, body_cx, body_cy, 16, 14, {
        'light': CREAM_LIGHT, 'main': CREAM, 'dark': CREAM_DARK, 'outline': OUTLINE
    })

    # ── 2. Head (overlapping top of body) ──
    head_cx, head_cy = bird_cx, bird_cy - 8
    fill_ellipse(draw, head_cx, head_cy, 13, 11, {
        'light': CREAM_LIGHT, 'main': CREAM, 'dark': CREAM_DARK, 'outline': OUTLINE
    })

    # ── 3. Yellow face area (heart-shaped region around beak/cheeks) ──
    # Large yellow patch covering lower face
    face_cx, face_cy = head_cx, head_cy + 2
    fill_ellipse(draw, face_cx, face_cy, 10, 7, {
        'light': YELLOW, 'main': YELLOW_FACE, 'dark': YELLOW_DARK, 'outline': TRANSPARENT
    })

    # ── 4. Crest (flame-like yellow feathers on top) ──
    # Main flame shape - pointed upward with irregular edges
    crest_base_y = head_cy - 8
    # Draw flame polygon
    flame_points = [
        (head_cx - 4, crest_base_y + 2),
        (head_cx - 6, crest_base_y - 2),
        (head_cx - 3, crest_base_y - 4),
        (head_cx - 5, crest_base_y - 8),
        (head_cx - 2, crest_base_y - 6),
        (head_cx - 1, crest_base_y - 12),
        (head_cx + 1, crest_base_y - 8),
        (head_cx + 3, crest_base_y - 10),
        (head_cx + 4, crest_base_y - 6),
        (head_cx + 5, crest_base_y - 8),
        (head_cx + 6, crest_base_y - 4),
        (head_cx + 3, crest_base_y - 2),
        (head_cx + 5, crest_base_y + 1),
        (head_cx + 2, crest_base_y + 2),
    ]
    draw.polygon(flame_points, fill=YELLOW)
    # Crest outline
    for i in range(len(flame_points)):
        p1 = flame_points[i]
        p2 = flame_points[(i + 1) % len(flame_points)]
        draw.line([p1, p2], fill=OUTLINE, width=1)
    # Inner flame detail (darker yellow streaks)
    draw.line([(head_cx - 2, crest_base_y - 3), (head_cx - 3, crest_base_y - 7)],
              fill=YELLOW_DARK, width=1)
    draw.line([(head_cx + 2, crest_base_y - 4), (head_cx + 1, crest_base_y - 9)],
              fill=YELLOW_DARK, width=1)

    # ── 5. Eyes ──
    eye_l_x, eye_r_x = head_cx - 4, head_cx + 3
    eye_y = head_cy - 1

    if eye_state == "open":
        # Left eye
        draw.ellipse([(eye_l_x - 2, eye_y - 2), (eye_l_x + 2, eye_y + 2)],
                     fill=EYE_BLACK)
        px(draw, eye_l_x - 1, eye_y - 1, EYE_WHITE)
        # Right eye
        draw.ellipse([(eye_r_x - 2, eye_y - 2), (eye_r_x + 2, eye_y + 2)],
                     fill=EYE_BLACK)
        px(draw, eye_r_x - 1, eye_y - 1, EYE_WHITE)
        # Look direction
        if look == "left":
            px(draw, eye_l_x - 2, eye_y, EYE_WHITE)
            px(draw, eye_r_x - 2, eye_y, EYE_WHITE)
        elif look == "right":
            px(draw, eye_l_x + 1, eye_y, EYE_WHITE)
            px(draw, eye_r_x + 1, eye_y, EYE_WHITE)
        elif look == "up":
            px(draw, eye_l_x, eye_y - 2, EYE_WHITE)
            px(draw, eye_r_x, eye_y - 2, EYE_WHITE)

    elif eye_state == "closed":
        draw.line([(eye_l_x - 2, eye_y), (eye_l_x + 2, eye_y)], fill=OUTLINE, width=1)
        draw.line([(eye_r_x - 2, eye_y), (eye_r_x + 2, eye_y)], fill=OUTLINE, width=1)

    elif eye_state == "half":
        draw.line([(eye_l_x - 2, eye_y), (eye_l_x + 2, eye_y)], fill=OUTLINE, width=1)
        px(draw, eye_l_x, eye_y - 1, EYE_BLACK)
        draw.line([(eye_r_x - 2, eye_y), (eye_r_x + 2, eye_y)], fill=OUTLINE, width=1)
        px(draw, eye_r_x, eye_y - 1, EYE_BLACK)

    elif eye_state == "wide":
        draw.ellipse([(eye_l_x - 3, eye_y - 3), (eye_l_x + 3, eye_y + 3)],
                     fill=EYE_BLACK)
        px(draw, eye_l_x - 1, eye_y - 2, EYE_WHITE)
        draw.ellipse([(eye_r_x - 3, eye_y - 3), (eye_r_x + 3, eye_y + 3)],
                     fill=EYE_BLACK)
        px(draw, eye_r_x - 1, eye_y - 2, EYE_WHITE)

    elif eye_state == "x_eyes":
        for d in range(-2, 3):
            px(draw, eye_l_x + d, eye_y + d, RED)
            px(draw, eye_l_x + d, eye_y - d, RED)
            px(draw, eye_r_x + d, eye_y + d, RED)
            px(draw, eye_r_x + d, eye_y - d, RED)

    # ── 6. Beak (orange, triangular) ──
    beak_cx, beak_y = head_cx, head_cy + 3
    draw.polygon([
        (beak_cx - 2, beak_y),
        (beak_cx + 2, beak_y),
        (beak_cx + 1, beak_y + 3),
        (beak_cx - 1, beak_y + 3),
    ], fill=ORANGE)
    draw.polygon([
        (beak_cx - 2, beak_y),
        (beak_cx + 2, beak_y),
        (beak_cx + 1, beak_y + 3),
        (beak_cx - 1, beak_y + 3),
    ], outline=OUTLINE, width=1)
    # Beak highlight
    px(draw, beak_cx, beak_y + 1, ORANGE_DARK)

    if beak_open:
        draw.polygon([
            (beak_cx - 1, beak_y + 4),
            (beak_cx + 1, beak_y + 4),
            (beak_cx, beak_y + 6),
        ], fill=ORANGE_DARK)
        draw.polygon([
            (beak_cx - 1, beak_y + 4),
            (beak_cx + 1, beak_y + 4),
            (beak_cx, beak_y + 6),
        ], outline=OUTLINE, width=1)

    # ── 7. Pink cheeks ──
    blush_y = head_cy + 2
    draw.ellipse([(head_cx - 9, blush_y - 1), (head_cx - 6, blush_y + 2)],
                 fill=PINK)
    draw.ellipse([(head_cx + 6, blush_y - 1), (head_cx + 9, blush_y + 2)],
                 fill=PINK)

    # ── 8. Wings ──
    wing_y = body_cy - 4
    if wing_fold == "normal":
        # Folded wings against body sides
        draw.polygon([
            (body_cx - 14, wing_y), (body_cx - 16, wing_y + 8),
            (body_cx - 12, wing_y + 10), (body_cx - 11, wing_y + 2),
        ], fill=CREAM_DARK, outline=OUTLINE)
        draw.polygon([
            (body_cx + 14, wing_y), (body_cx + 16, wing_y + 8),
            (body_cx + 12, wing_y + 10), (body_cx + 11, wing_y + 2),
        ], fill=CREAM_DARK, outline=OUTLINE)
    elif wing_fold == "raised":
        draw.polygon([
            (body_cx - 14, wing_y), (body_cx - 18, wing_y - 6),
            (body_cx - 14, wing_y + 2), (body_cx - 11, wing_y + 4),
        ], fill=CREAM_DARK, outline=OUTLINE)
        draw.polygon([
            (body_cx + 14, wing_y), (body_cx + 18, wing_y - 6),
            (body_cx + 14, wing_y + 2), (body_cx + 11, wing_y + 4),
        ], fill=CREAM_DARK, outline=OUTLINE)
    elif wing_fold == "spread":
        draw.polygon([
            (body_cx - 14, wing_y), (body_cx - 22, wing_y - 4),
            (body_cx - 20, wing_y + 6), (body_cx - 11, wing_y + 6),
        ], fill=CREAM_DARK, outline=OUTLINE)
        draw.polygon([
            (body_cx + 14, wing_y), (body_cx + 22, wing_y - 4),
            (body_cx + 20, wing_y + 6), (body_cx + 11, wing_y + 6),
        ], fill=CREAM_DARK, outline=OUTLINE)
    elif wing_fold == "down":
        draw.polygon([
            (body_cx - 14, wing_y + 2), (body_cx - 16, wing_y + 10),
            (body_cx - 12, wing_y + 12), (body_cx - 11, wing_y + 4),
        ], fill=CREAM_DARK, outline=OUTLINE)
        draw.polygon([
            (body_cx + 14, wing_y + 2), (body_cx + 16, wing_y + 10),
            (body_cx + 12, wing_y + 12), (body_cx + 11, wing_y + 4),
        ], fill=CREAM_DARK, outline=OUTLINE)

    # ── 9. Feet ──
    feet_y = body_cy + 12
    # Left foot
    draw.line([(body_cx - 6, feet_y), (body_cx - 8, feet_y + 4)], fill=ORANGE, width=1)
    draw.line([(body_cx - 8, feet_y + 4), (body_cx - 10, feet_y + 4)], fill=ORANGE, width=1)
    draw.line([(body_cx - 8, feet_y + 4), (body_cx - 7, feet_y + 5)], fill=ORANGE, width=1)
    px(draw, body_cx - 6, feet_y, OUTLINE)
    # Right foot
    draw.line([(body_cx + 6, feet_y), (body_cx + 8, feet_y + 4)], fill=ORANGE, width=1)
    draw.line([(body_cx + 8, feet_y + 4), (body_cx + 10, feet_y + 4)], fill=ORANGE, width=1)
    draw.line([(body_cx + 8, feet_y + 4), (body_cx + 7, feet_y + 5)], fill=ORANGE, width=1)
    px(draw, body_cx + 6, feet_y, OUTLINE)


# ── Accessory Drawing Helpers ──

def draw_zzz(draw, frame, ox=0, oy=0):
    """Floating Zzz letters that rise and fade."""
    import math
    for i in range(3):
        progress = (frame + i * 2) % 8
        zx = 44 + ox + i * 3
        zy = 14 + oy - progress * 2
        alpha = max(0, 255 - progress * 40)
        if alpha > 0:
            color = (140, 190, 230, alpha)
            for dx in range(3):
                px(draw, zx + dx, zy, color)
                px(draw, zx + dx, zy + 3, color)
            px(draw, zx + 2, zy + 1, color)
            px(draw, zx + 1, zy + 2, color)


def draw_thought_bubble(draw, ox=0, oy=0, symbol="?"):
    """Thought bubble with ? or !."""
    bx, by = 42 + ox, 4 + oy
    draw.ellipse([(38+ox, 14+oy), (40+ox, 16+oy)], fill=WHITE, outline=OUTLINE)
    draw.ellipse([(40+ox, 10+oy), (43+ox, 13+oy)], fill=WHITE, outline=OUTLINE)
    draw.ellipse([(bx-1, by-1), (bx+13, by+11)], fill=WHITE, outline=OUTLINE)
    if symbol == "?":
        draw.text((bx+3, by+1), "?", fill=PURPLE)
    else:
        draw.text((bx+3, by+1), "!", fill=RED)


def draw_music_note(draw, x, y, frame):
    """Small music note."""
    ny = y - (frame % 3)
    draw.ellipse([(x, ny+2), (x+2, ny+4)], fill=PURPLE)
    draw.line([(x+2, ny), (x+2, ny+3)], fill=PURPLE, width=1)
    px(draw, x+3, ny, PURPLE)
    px(draw, x+3, ny+1, PURPLE)


def draw_sparkle(draw, x, y, frame):
    """Twinkling star sparkle."""
    s = frame % 4
    color = STAR_YELLOW
    if s == 0:
        px(draw, x, y, color)
    elif s == 1:
        draw.line([(x-1, y), (x+1, y)], fill=color, width=1)
        draw.line([(x, y-1), (x, y+1)], fill=color, width=1)
    elif s == 2:
        draw.line([(x-2, y), (x+2, y)], fill=color, width=1)
        draw.line([(x, y-2), (x, y+2)], fill=color, width=1)
    else:
        px(draw, x-1, y, color)
        px(draw, x+1, y, color)
        px(draw, x, y-1, color)
        px(draw, x, y+1, color)


def draw_exclamation(draw, x, y, bounce=0):
    """Bouncing exclamation mark."""
    ey = y - bounce
    draw.rectangle([(x, ey), (x+1, ey+4)], fill=RED)
    px(draw, x, ey+6, RED)


def draw_error_sparks(draw, frame, ox=0, oy=0):
    """Error sparks around the bird."""
    positions = [(12, 10), (50, 12), (10, 40), (52, 38)]
    for i, (sx, sy) in enumerate(positions):
        s = (frame + i) % 4
        color = RED_LIGHT
        if s == 0:
            px(draw, sx+ox, sy+oy, color)
        elif s == 1:
            draw.line([(sx-1+ox, sy+oy), (sx+1+ox, sy+oy)], fill=color, width=1)
            draw.line([(sx+ox, sy-1+oy), (sx+ox, sy+1+oy)], fill=color, width=1)
        elif s == 2:
            draw.line([(sx-2+ox, sy+oy), (sx+2+ox, sy+oy)], fill=color, width=1)
        else:
            px(draw, sx+ox, sy-1+oy, color)
            px(draw, sx+ox, sy+1+oy, color)


def draw_keyboard(draw, ox=0, oy=0):
    """Tiny keyboard under the bird."""
    kx, ky = 22 + ox, 48 + oy
    draw.rectangle([(kx, ky), (kx + 16, ky + 5)], fill=(100, 100, 110, 255), outline=OUTLINE)
    for row in range(2):
        for col in range(4):
            draw.rectangle(
                [(kx + 1 + col * 4, ky + 1 + row * 2),
                 (kx + 3 + col * 4, ky + 2 + row * 2)],
                fill=WHITE
            )


def draw_sweat_drop(draw, x, y, frame):
    """Nervous sweat drop."""
    drop_y = y + (frame % 3) * 2
    draw.ellipse([(x, drop_y), (x+2, drop_y+3)], fill=LIGHT_BLUE)
    px(draw, x+1, drop_y-1, LIGHT_BLUE)


# ── Sprite Sheet Generators ──

def generate_sleeping(path):
    """6 frames: sleeping cockatiel with breathing and floating Zzz."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 6, FRAME_SIZE), (0, 0, 0, 0))
    for f in range(6):
        frame = Image.new("RGBA", (FRAME_SIZE, FRAME_SIZE), (0, 0, 0, 0))
        draw = ImageDraw.Draw(frame)
        bob = 1 if f in (1, 4) else 0
        draw_base_bird(draw, oy=bob, eye_state="closed", wing_fold="normal")
        draw_zzz(draw, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*6}x{FRAME_SIZE})")


def generate_waiting(path):
    """8 frames: idle bird looking around, occasional blink."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 8, FRAME_SIZE), (0, 0, 0, 0))
    looks = ["center", "left", "left", "center", "right", "right", "center", "center"]
    blinks = [False, False, False, True, False, False, False, True]
    for f in range(8):
        frame = Image.new("RGBA", (FRAME_SIZE, FRAME_SIZE), (0, 0, 0, 0))
        draw = ImageDraw.Draw(frame)
        draw_base_bird(draw, eye_state="closed" if blinks[f] else "open",
                       look=looks[f], wing_fold="normal")
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*8}x{FRAME_SIZE})")


def generate_thinking(path):
    """6 frames: bird looking up with thought bubble."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 6, FRAME_SIZE), (0, 0, 0, 0))
    for f in range(6):
        frame = Image.new("RGBA", (FRAME_SIZE, FRAME_SIZE), (0, 0, 0, 0))
        draw = ImageDraw.Draw(frame)
        draw_base_bird(draw, eye_state="open", look="up", wing_fold="normal")
        symbol = "?" if f < 3 else "!"
        draw_thought_bubble(draw, symbol=symbol)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*6}x{FRAME_SIZE})")


def generate_working(path):
    """8 frames: bird actively pecking/typing."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 8, FRAME_SIZE), (0, 0, 0, 0))
    for f in range(8):
        frame = Image.new("RGBA", (FRAME_SIZE, FRAME_SIZE), (0, 0, 0, 0))
        draw = ImageDraw.Draw(frame)
        bob = 2 if f % 2 == 0 else 0
        open_beak = f % 2 == 0
        draw_base_bird(draw, oy=bob, eye_state="open",
                       beak_open=open_beak, wing_fold="normal")
        draw_keyboard(draw, oy=bob)
        if f in (1, 3, 5, 7):
            draw_sparkle(draw, 12, 14, f)
            draw_sparkle(draw, 50, 16, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*8}x{FRAME_SIZE})")


def generate_pending_approval(path):
    """6 frames: hesitant bird waiting for approval."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 6, FRAME_SIZE), (0, 0, 0, 0))
    for f in range(6):
        frame = Image.new("RGBA", (FRAME_SIZE, FRAME_SIZE), (0, 0, 0, 0))
        draw = ImageDraw.Draw(frame)
        bob = 1 if f in (2, 5) else 0
        draw_base_bird(draw, oy=bob, eye_state="wide", wing_fold="normal")
        bounce = abs((f % 4) - 2)
        draw_exclamation(draw, 48, 10, bounce)
        if f % 2 == 0:
            draw_sweat_drop(draw, 42, 14, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*6}x{FRAME_SIZE})")


def generate_notify(path):
    """6 frames: excited bird with music notes and sparkles."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 6, FRAME_SIZE), (0, 0, 0, 0))
    for f in range(6):
        frame = Image.new("RGBA", (FRAME_SIZE, FRAME_SIZE), (0, 0, 0, 0))
        draw = ImageDraw.Draw(frame)
        jump = -3 if f in (1, 2, 3) else 0
        draw_base_bird(draw, oy=jump, eye_state="wide",
                       beak_open=True,
                       wing_fold="spread" if f in (1, 2, 3) else "normal")
        draw_music_note(draw, 10, 12 + jump, f)
        draw_music_note(draw, 50, 10 + jump, (f + 2) % 6)
        draw_sparkle(draw, 8, 30 + jump, f)
        draw_sparkle(draw, 54, 28 + jump, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*6}x{FRAME_SIZE})")


def generate_subagent(path):
    """8 frames: main bird with a small companion bird."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 8, FRAME_SIZE), (0, 0, 0, 0))
    for f in range(8):
        frame = Image.new("RGBA", (FRAME_SIZE, FRAME_SIZE), (0, 0, 0, 0))
        draw = ImageDraw.Draw(frame)
        # Main bird slightly to the left
        draw_base_bird(draw, ox=-4, oy=0, eye_state="open", wing_fold="normal")
        # Small companion bird (mini version, top-right)
        sx, sy = 44, 8
        # Mini body
        draw.ellipse([(sx, sy+4), (sx+10, sy+12)], fill=CREAM, outline=OUTLINE)
        # Mini crest
        draw.polygon([(sx+4, sy+4), (sx+3, sy), (sx+5, sy+2), (sx+7, sy)],
                     fill=YELLOW)
        # Mini eye
        px(draw, sx+3, sy+7, EYE_BLACK)
        px(draw, sx+3, sy+6, EYE_WHITE)
        # Mini beak
        px(draw, sx+6, sy+8, ORANGE)
        px(draw, sx+7, sy+8, ORANGE)
        # Mini blush
        px(draw, sx+2, sy+8, PINK)
        px(draw, sx+8, sy+8, PINK)
        # Connection sparkle
        draw_sparkle(draw, 40, 14, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*8}x{FRAME_SIZE})")


def generate_error(path):
    """4 frames: startled/error bird with X eyes and sparks."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 4, FRAME_SIZE), (0, 0, 0, 0))
    for f in range(4):
        frame = Image.new("RGBA", (FRAME_SIZE, FRAME_SIZE), (0, 0, 0, 0))
        draw = ImageDraw.Draw(frame)
        shake_x = 1 if f % 2 == 0 else -1
        draw_base_bird(draw, ox=shake_x, eye_state="x_eyes",
                       beak_open=True, wing_fold="raised")
        draw_error_sparks(draw, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*4}x{FRAME_SIZE})")


def generate_stopped(path):
    """4 frames: bird settling down."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 4, FRAME_SIZE), (0, 0, 0, 0))
    for f in range(4):
        frame = Image.new("RGBA", (FRAME_SIZE, FRAME_SIZE), (0, 0, 0, 0))
        draw = ImageDraw.Draw(frame)
        eye = "closed" if f >= 2 else "half"
        draw_base_bird(draw, eye_state=eye, wing_fold="normal")
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*4}x{FRAME_SIZE})")


# ── Main ──

def main():
    output_dir = os.path.join(os.path.dirname(os.path.dirname(
        os.path.abspath(__file__))), "assets", "sprites")
    os.makedirs(output_dir, exist_ok=True)

    print("🎨 Generating cockatiel sprite sheets (pixel art style)...\n")

    generate_sleeping(os.path.join(output_dir, "sleeping.png"))
    generate_waiting(os.path.join(output_dir, "waiting.png"))
    generate_thinking(os.path.join(output_dir, "thinking.png"))
    generate_working(os.path.join(output_dir, "working.png"))
    generate_pending_approval(os.path.join(output_dir, "pending_approval.png"))
    generate_notify(os.path.join(output_dir, "notify.png"))
    generate_subagent(os.path.join(output_dir, "subagent.png"))
    generate_error(os.path.join(output_dir, "error.png"))
    generate_stopped(os.path.join(output_dir, "stopped.png"))

    print(f"\n✅ All sprites saved to: {output_dir}")
    print("   Run 'cargo run' to see the pet in action!")


if __name__ == "__main__":
    main()
