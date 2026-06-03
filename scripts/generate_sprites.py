#!/usr/bin/env python3
"""Generate pixel art sprite sheets for the Claude Poke cockatiel pet.

Style: cute pixel art cockatiel (参考玄凤鹦鹉像素图)
Each frame: 64x64 pixels, RGBA, transparent background.
Frames arranged as horizontal strip in PNG.
"""

from PIL import Image, ImageDraw
import os

# ── Color Palette (matching reference image) ──
# Bird colors
CREAM       = (245, 230, 200, 255)   # Body main
CREAM_LIGHT = (255, 245, 225, 255)   # Body highlight
CREAM_DARK  = (220, 200, 165, 255)   # Body shadow
YELLOW      = (245, 197, 24, 255)    # Crest bright
YELLOW_DARK = (232, 168, 0, 255)     # Crest shadow
ORANGE      = (232, 120, 48, 255)    # Beak & feet
ORANGE_DARK = (200, 96, 32, 255)     # Beak shadow
PINK        = (245, 160, 160, 255)   # Cheek blush
PINK_LIGHT  = (255, 180, 180, 255)   # Blush highlight

# Outline & detail
OUTLINE     = (26, 26, 46, 255)      # Dark navy-black outline
EYE_BLACK   = (26, 26, 46, 255)      # Eye color
EYE_WHITE   = (255, 255, 255, 255)   # Eye highlight

# Accessories
WHITE       = (255, 255, 255, 255)
RED         = (230, 60, 60, 255)
RED_LIGHT   = (255, 80, 80, 255)
PURPLE      = (160, 100, 220, 255)
LIGHT_BLUE  = (140, 190, 230, 255)
STAR_YELLOW = (255, 220, 60, 255)

TRANSPARENT = (0, 0, 0, 0)
FRAME_SIZE = 64


def new_frame():
    return Image.new("RGBA", (FRAME_SIZE, FRAME_SIZE), TRANSPARENT)


def px(draw, x, y, color):
    """Draw a single pixel."""
    if 0 <= x < FRAME_SIZE and 0 <= y < FRAME_SIZE:
        draw.point((x, y), fill=color)


def rect(draw, x1, y1, x2, y2, color):
    """Draw a filled rectangle."""
    draw.rectangle([(x1, y1), (x2, y2)], fill=color)


def fill_area(draw, pixels, colors, ox=0, oy=0):
    """Draw a list of (x, y, color_key) pixels."""
    for x, y, ck in pixels:
        px(draw, x + ox, y + oy, colors.get(ck, TRANSPARENT))


# ── Base Cockatiel Drawing ──

def draw_base_bird(draw, ox=0, oy=0, eye_state="open", look="center",
                   beak_open=False, wing_fold="normal", head_tilt=0):
    """Draw the base cockatiel at 64x64.

    eye_state: "open", "closed", "half", "wide", "x_eyes"
    look: "center", "left", "right", "up"
    wing_fold: "normal", "raised", "spread", "down"
    head_tilt: pixels to shift head vertically
    """
    c, cl, cd = CREAM, CREAM_LIGHT, CREAM_DARK
    o = OUTLINE

    # ── Body outline (filled) ──
    # Main body - round shape
    body_pixels = []
    for y in range(24 + oy, 50 + oy):
        for x in range(16 + ox, 48 + ox):
            # Calculate distance from center of body
            cx, cy = 32 + ox, 36 + oy
            dx = (x - cx) / 14.0
            dy = (y - cy) / 12.0
            if dx * dx + dy * dy <= 1.0:
                # Shading: lighter on top-left, darker on bottom-right
                shade = dx + dy
                if shade < -0.3:
                    color = cl
                elif shade > 0.3:
                    color = cd
                else:
                    color = c
                body_pixels.append((x, y, color))

    for x, y, color in body_pixels:
        px(draw, x, y, color)

    # ── Body outline ──
    for y in range(23 + oy, 51 + oy):
        for x in range(15 + ox, 49 + ox):
            cx, cy = 32 + ox, 36 + oy
            dx = (x - cx) / 15.0
            dy = (y - cy) / 13.0
            dist = dx * dx + dy * dy
            if 0.92 < dist <= 1.08:
                px(draw, x, y, o)

    # ── Head (slightly overlapping body) ──
    head_cx, head_cy = 32 + ox, 24 + oy + head_tilt
    for y in range(head_cy - 10, head_cy + 10):
        for x in range(head_cx - 10, head_cx + 11):
            dx = (x - head_cx) / 10.0
            dy = (y - head_cy) / 9.0
            if dx * dx + dy * dy <= 1.0:
                shade = dx + dy
                if shade < -0.3:
                    color = cl
                elif shade > 0.3:
                    color = cd
                else:
                    color = c
                px(draw, x, y, color)

    # Head outline
    for y in range(head_cy - 11, head_cy + 11):
        for x in range(head_cx - 11, head_cx + 12):
            dx = (x - head_cx) / 11.0
            dy = (y - head_cy) / 10.0
            dist = dx * dx + dy * dy
            if 0.88 < dist <= 1.12:
                px(draw, x, y, o)

    # ── Crest (yellow feathers on top) ──
    crest_points = [
        (head_cx - 2, head_cy - 9),
        (head_cx - 5, head_cy - 16),
        (head_cx - 2, head_cy - 13),
        (head_cx, head_cy - 18),
        (head_cx + 2, head_cy - 14),
        (head_cx + 4, head_cy - 17),
        (head_cx + 5, head_cy - 12),
        (head_cx + 3, head_cy - 9),
    ]
    draw.polygon(crest_points, fill=YELLOW)
    # Crest outline
    for i in range(len(crest_points)):
        p1 = crest_points[i]
        p2 = crest_points[(i + 1) % len(crest_points)]
        draw.line([p1, p2], fill=OUTLINE, width=1)
    # Crest highlight
    draw.line([(head_cx - 1, head_cy - 14), (head_cx + 1, head_cy - 17)],
              fill=YELLOW_DARK, width=1)

    # ── Eyes ──
    eye_l_x, eye_r_x = head_cx - 5, head_cx + 4
    eye_y = head_cy - 1 + head_tilt

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
        # Closed eyes (horizontal lines)
        draw.line([(eye_l_x - 2, eye_y), (eye_l_x + 2, eye_y)], fill=o, width=1)
        draw.line([(eye_r_x - 2, eye_y), (eye_r_x + 2, eye_y)], fill=o, width=1)

    elif eye_state == "half":
        # Half-closed eyes
        draw.line([(eye_l_x - 2, eye_y), (eye_l_x + 2, eye_y)], fill=o, width=1)
        px(draw, eye_l_x, eye_y - 1, EYE_BLACK)
        draw.line([(eye_r_x - 2, eye_y), (eye_r_x + 2, eye_y)], fill=o, width=1)
        px(draw, eye_r_x, eye_y - 1, EYE_BLACK)

    elif eye_state == "wide":
        # Wide open eyes (bigger)
        draw.ellipse([(eye_l_x - 3, eye_y - 3), (eye_l_x + 3, eye_y + 3)],
                     fill=EYE_BLACK)
        px(draw, eye_l_x - 1, eye_y - 2, EYE_WHITE)
        draw.ellipse([(eye_r_x - 3, eye_y - 3), (eye_r_x + 3, eye_y + 3)],
                     fill=EYE_BLACK)
        px(draw, eye_r_x - 1, eye_y - 2, EYE_WHITE)

    elif eye_state == "x_eyes":
        # X eyes for error
        for dx in range(-2, 3):
            px(draw, eye_l_x + dx, eye_y + dx, RED)
            px(draw, eye_l_x + dx, eye_y - dx, RED)
            px(draw, eye_r_x + dx, eye_y + dx, RED)
            px(draw, eye_r_x + dx, eye_y - dx, RED)

    # ── Beak ──
    beak_cx, beak_y = head_cx, head_cy + 4 + head_tilt
    # Upper beak
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

    if beak_open:
        # Lower beak
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

    # ── Cheek blush ──
    blush_y = head_cy + 1 + head_tilt
    draw.ellipse([(head_cx - 9, blush_y - 1), (head_cx - 6, blush_y + 2)],
                 fill=PINK)
    draw.ellipse([(head_cx + 6, blush_y - 1), (head_cx + 9, blush_y + 2)],
                 fill=PINK)

    # ── Wings ──
    wing_y = 32 + oy
    if wing_fold == "normal":
        # Folded wings against body
        draw.polygon([
            (18 + ox, wing_y), (15 + ox, wing_y + 8),
            (20 + ox, wing_y + 10), (22 + ox, wing_y + 2),
        ], fill=CREAM_DARK)
        draw.polygon([
            (46 + ox, wing_y), (49 + ox, wing_y + 8),
            (44 + ox, wing_y + 10), (42 + ox, wing_y + 2),
        ], fill=CREAM_DARK)
        # Wing outline
        draw.line([(18+ox, wing_y), (15+ox, wing_y+8)], fill=o, width=1)
        draw.line([(15+ox, wing_y+8), (20+ox, wing_y+10)], fill=o, width=1)
        draw.line([(46+ox, wing_y), (49+ox, wing_y+8)], fill=o, width=1)
        draw.line([(49+ox, wing_y+8), (44+ox, wing_y+10)], fill=o, width=1)

    elif wing_fold == "raised":
        draw.polygon([
            (18 + ox, wing_y), (12 + ox, wing_y - 6),
            (14 + ox, wing_y + 2), (20 + ox, wing_y + 4),
        ], fill=CREAM_DARK)
        draw.polygon([
            (46 + ox, wing_y), (52 + ox, wing_y - 6),
            (50 + ox, wing_y + 2), (44 + ox, wing_y + 4),
        ], fill=CREAM_DARK)
        draw.line([(18+ox, wing_y), (12+ox, wing_y-6)], fill=o, width=1)
        draw.line([(46+ox, wing_y), (52+ox, wing_y-6)], fill=o, width=1)

    elif wing_fold == "spread":
        draw.polygon([
            (18 + ox, wing_y), (8 + ox, wing_y - 4),
            (10 + ox, wing_y + 6), (20 + ox, wing_y + 6),
        ], fill=CREAM_DARK)
        draw.polygon([
            (46 + ox, wing_y), (56 + ox, wing_y - 4),
            (54 + ox, wing_y + 6), (44 + ox, wing_y + 6),
        ], fill=CREAM_DARK)
        draw.line([(18+ox, wing_y), (8+ox, wing_y-4)], fill=o, width=1)
        draw.line([(46+ox, wing_y), (56+ox, wing_y-4)], fill=o, width=1)

    elif wing_fold == "down":
        draw.polygon([
            (18 + ox, wing_y + 2), (14 + ox, wing_y + 10),
            (20 + ox, wing_y + 12), (22 + ox, wing_y + 4),
        ], fill=CREAM_DARK)
        draw.polygon([
            (46 + ox, wing_y + 2), (50 + ox, wing_y + 10),
            (44 + ox, wing_y + 12), (42 + ox, wing_y + 4),
        ], fill=CREAM_DARK)

    # ── Feet ──
    feet_y = 48 + oy
    # Left foot
    draw.line([(26 + ox, feet_y), (24 + ox, feet_y + 4)], fill=ORANGE, width=1)
    draw.line([(24 + ox, feet_y + 4), (22 + ox, feet_y + 4)], fill=ORANGE, width=1)
    draw.line([(24 + ox, feet_y + 4), (25 + ox, feet_y + 5)], fill=ORANGE, width=1)
    px(draw, 26 + ox, feet_y, OUTLINE)
    # Right foot
    draw.line([(38 + ox, feet_y), (40 + ox, feet_y + 4)], fill=ORANGE, width=1)
    draw.line([(40 + ox, feet_y + 4), (42 + ox, feet_y + 4)], fill=ORANGE, width=1)
    draw.line([(40 + ox, feet_y + 4), (39 + ox, feet_y + 5)], fill=ORANGE, width=1)
    px(draw, 38 + ox, feet_y, OUTLINE)


# ── Accessory Drawing Helpers ──

def draw_zzz(draw, frame, ox=0, oy=0):
    """Floating Zzz letters that rise and fade."""
    for i in range(3):
        progress = (frame + i * 2) % 8
        zx = 44 + ox + i * 3
        zy = 18 + oy - progress * 2
        alpha = max(0, 255 - progress * 40)
        if alpha > 0:
            color = (140, 190, 230, alpha)
            # Draw pixel Z
            for dx in range(3):
                px(draw, zx + dx, zy, color)
                px(draw, zx + dx, zy + 3, color)
            px(draw, zx + 2, zy + 1, color)
            px(draw, zx + 1, zy + 2, color)


def draw_thought_bubble(draw, ox=0, oy=0, symbol="?"):
    """Thought bubble with ? or !."""
    bx, by = 42 + ox, 6 + oy
    # Bubble dots
    draw.ellipse([(38+ox, 16+oy), (40+ox, 18+oy)], fill=WHITE, outline=OUTLINE)
    draw.ellipse([(40+ox, 12+oy), (43+ox, 15+oy)], fill=WHITE, outline=OUTLINE)
    # Main bubble
    draw.ellipse([(bx-1, by-1), (bx+13, by+11)], fill=WHITE, outline=OUTLINE)
    # Symbol
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


def draw_question_bubble(draw, x, y, frame):
    """Question mark in a small bubble."""
    draw.ellipse([(x-1, y-1), (x+9, y+9)], fill=WHITE, outline=OUTLINE)
    draw.text((x+2, y), "?", fill=PURPLE)


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
    # Keys
    for row in range(2):
        for col in range(4):
            draw.rectangle(
                [(kx + 1 + col * 4, ky + 1 + row * 2),
                 (kx + 3 + col * 4, ky + 2 + row * 2)],
                fill=WHITE
            )


# ── Sprite Sheet Generators ──

def generate_sleeping(path):
    """6 frames: sleeping cockatiel with breathing and floating Zzz."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 6, FRAME_SIZE), TRANSPARENT)
    for f in range(6):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        # Gentle breathing bob
        bob = 1 if f in (1, 4) else 0
        draw_base_bird(draw, oy=bob, eye_state="closed", wing_fold="normal")
        draw_zzz(draw, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*6}x{FRAME_SIZE})")


def generate_waiting(path):
    """8 frames: idle bird looking around, occasional blink."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 8, FRAME_SIZE), TRANSPARENT)
    looks = ["center", "left", "left", "center", "right", "right", "center", "center"]
    blinks = [False, False, False, True, False, False, False, True]
    for f in range(8):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        draw_base_bird(draw, eye_state="closed" if blinks[f] else "open",
                       look=looks[f], wing_fold="normal")
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*8}x{FRAME_SIZE})")


def generate_thinking(path):
    """6 frames: bird looking up with thought bubble."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 6, FRAME_SIZE), TRANSPARENT)
    for f in range(6):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        # Slight head tilt up
        draw_base_bird(draw, eye_state="open", look="up",
                       wing_fold="normal", head_tilt=-1)
        # Thought bubble alternates ? and !
        symbol = "?" if f < 3 else "!"
        draw_thought_bubble(draw, symbol=symbol)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*6}x{FRAME_SIZE})")


def generate_working(path):
    """8 frames: bird actively pecking/typing."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 8, FRAME_SIZE), TRANSPARENT)
    for f in range(8):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        # Bobbing head motion
        bob = 2 if f % 2 == 0 else 0
        open_beak = f % 2 == 0
        draw_base_bird(draw, oy=bob, eye_state="open",
                       beak_open=open_beak, wing_fold="normal")
        draw_keyboard(draw, oy=bob)
        # Sparkles when active
        if f in (1, 3, 5, 7):
            draw_sparkle(draw, 12, 14, f)
            draw_sparkle(draw, 50, 16, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*8}x{FRAME_SIZE})")


def generate_pending_approval(path):
    """6 frames: hesitant bird waiting for approval."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 6, FRAME_SIZE), TRANSPARENT)
    for f in range(6):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        # Nervous bob
        bob = 1 if f in (2, 5) else 0
        draw_base_bird(draw, oy=bob, eye_state="wide", wing_fold="normal")
        # Bouncing exclamation
        bounce = abs((f % 4) - 2)
        draw_exclamation(draw, 48, 10, bounce)
        # Nervous sweat drop
        if f % 2 == 0:
            drop_y = 16 + (f % 3) * 2
            draw.ellipse([(42, drop_y), (44, drop_y+2)], fill=LIGHT_BLUE)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*6}x{FRAME_SIZE})")


def generate_notify(path):
    """6 frames: excited bird with music notes and sparkles."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 6, FRAME_SIZE), TRANSPARENT)
    for f in range(6):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        # Jumping motion
        jump = -3 if f in (1, 2, 3) else 0
        draw_base_bird(draw, oy=jump, eye_state="wide",
                       beak_open=True, wing_fold="spread" if f in (1, 2, 3) else "normal")
        # Music notes and sparkles
        draw_music_note(draw, 10, 12 + jump, f)
        draw_music_note(draw, 50, 10 + jump, (f + 2) % 6)
        draw_sparkle(draw, 8, 30 + jump, f)
        draw_sparkle(draw, 54, 28 + jump, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*6}x{FRAME_SIZE})")


def generate_subagent(path):
    """8 frames: main bird with a small companion bird."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 8, FRAME_SIZE), TRANSPARENT)
    for f in range(8):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        # Main bird slightly to the left
        draw_base_bird(draw, ox=-4, oy=0, eye_state="open",
                       wing_fold="normal")
        # Small companion bird (tiny version, top-right)
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
        # Connection line (dotted)
        if f % 2 == 0:
            draw.line([(38, 20), (sx, sy+6)], fill=LIGHT_BLUE, width=1)
        # Sparkles between them
        draw_sparkle(draw, 40, 14, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*8}x{FRAME_SIZE})")


def generate_error(path):
    """4 frames: startled/error bird with X eyes and sparks."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 4, FRAME_SIZE), TRANSPARENT)
    for f in range(4):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        # Shake effect
        shake_x = 1 if f % 2 == 0 else -1
        draw_base_bird(draw, ox=shake_x, eye_state="x_eyes",
                       beak_open=True, wing_fold="raised")
        draw_error_sparks(draw, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  ✓ {os.path.basename(path)} ({FRAME_SIZE*4}x{FRAME_SIZE})")


def generate_stopped(path):
    """4 frames: bird settling down."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 4, FRAME_SIZE), TRANSPARENT)
    for f in range(4):
        frame = new_frame()
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
