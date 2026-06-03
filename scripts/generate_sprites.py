#!/usr/bin/env python3
"""Generate pixel art sprite sheets for the Claude Poke cockatiel pet."""

from PIL import Image, ImageDraw
import os

# Colors from design doc
CREAM = (255, 248, 231, 255)      # Body #FFF8E7
YELLOW = (255, 215, 0, 255)       # Crest #FFD700
PINK = (255, 153, 153, 255)       # Blush #FF9999
GRAY = (224, 224, 224, 255)       # Wings #E0E0E0
BLACK = (0, 0, 0, 255)
WHITE = (255, 255, 255, 255)
ORANGE = (255, 165, 0, 255)       # Beak
DARK_GRAY = (160, 160, 160, 255)  # Wing detail
LIGHT_BLUE = (173, 216, 230, 255) # Sleeping ZZZ
RED = (255, 80, 80, 255)          # Error/Notify
PURPLE = (180, 130, 255, 255)     # Thinking bubble
TRANSPARENT = (0, 0, 0, 0)

FRAME_SIZE = 64


def new_frame():
    """Create a new transparent 64x64 frame."""
    return Image.new("RGBA", (FRAME_SIZE, FRAME_SIZE), TRANSPARENT)


def draw_bird_body(draw, ox=0, oy=0, scale=1):
    """Draw the basic cockatiel body shape."""
    s = scale
    # Body (oval)
    for y in range(-10*s, 12*s):
        for x in range(-8*s, 9*s):
            if (x/(8*s))**2 + (y/(12*s))**2 <= 1:
                px, py = 28*s + ox + x, 32*s + oy + y
                if 0 <= px < FRAME_SIZE and 0 <= py < FRAME_SIZE:
                    draw.point((px, py), fill=CREAM)
    return draw


def draw_crest(draw, offset_x=0, offset_y=0):
    """Draw the yellow crest on top of the head."""
    # Crest feathers (3 spikes)
    points = [
        (28 + offset_x, 18 + offset_y),
        (25 + offset_x, 8 + offset_y),
        (28 + offset_x, 12 + offset_y),
        (31 + offset_x, 6 + offset_y),
        (32 + offset_x, 14 + offset_y),
        (35 + offset_x, 10 + offset_y),
        (31 + offset_x, 18 + offset_y),
    ]
    draw.polygon(points, fill=YELLOW)
    return draw


def draw_eyes(draw, blink=False, looking="center"):
    """Draw eyes. looking: 'center', 'left', 'right', 'up'."""
    if blink:
        # Closed eyes (lines)
        draw.line([(24, 26), (26, 26)], fill=BLACK, width=1)
        draw.line([(31, 26), (33, 26)], fill=BLACK, width=1)
    else:
        # Open eyes
        # Left eye
        draw.ellipse([(24, 24), (27, 28)], fill=BLACK)
        draw.point((25, 25), fill=WHITE)  # Highlight
        # Right eye
        draw.ellipse([(30, 24), (33, 28)], fill=BLACK)
        draw.point((31, 25), fill=WHITE)  # Highlight

        # Pupil direction
        if looking == "left":
            draw.point((24, 26), fill=WHITE)
            draw.point((30, 26), fill=WHITE)
        elif looking == "right":
            draw.point((26, 26), fill=WHITE)
            draw.point((32, 26), fill=WHITE)
        elif looking == "up":
            draw.point((25, 24), fill=WHITE)
            draw.point((31, 24), fill=WHITE)
    return draw


def draw_beak(draw, open_beak=False):
    """Draw the orange beak."""
    # Upper beak
    draw.polygon([(28, 29), (29, 29), (30, 31), (27, 31)], fill=ORANGE)
    if open_beak:
        # Lower beak (slightly open)
        draw.polygon([(27, 32), (30, 32), (29, 33), (28, 33)], fill=ORANGE)
    return draw


def draw_blush(draw):
    """Draw pink cheek blush."""
    draw.ellipse([(21, 28), (24, 31)], fill=PINK)
    draw.ellipse([(33, 28), (36, 31)], fill=PINK)
    return draw


def draw_feet(draw):
    """Draw small orange feet."""
    # Left foot
    draw.line([(25, 44), (23, 48)], fill=ORANGE, width=1)
    draw.line([(23, 48), (21, 48)], fill=ORANGE, width=1)
    draw.line([(23, 48), (24, 49)], fill=ORANGE, width=1)
    # Right foot
    draw.line([(32, 44), (34, 48)], fill=ORANGE, width=1)
    draw.line([(34, 48), (36, 48)], fill=ORANGE, width=1)
    draw.line([(34, 48), (33, 49)], fill=ORANGE, width=1)
    return draw


def draw_wings(draw, spread=False, raised=False):
    """Draw wings."""
    if spread:
        # Spread wings
        draw.polygon([(18, 30), (10, 28), (12, 36), (20, 35)], fill=GRAY)
        draw.polygon([(39, 30), (47, 28), (45, 36), (37, 35)], fill=GRAY)
    elif raised:
        # Raised wings
        draw.polygon([(18, 28), (14, 20), (16, 28), (20, 32)], fill=GRAY)
        draw.polygon([(39, 28), (43, 20), (41, 28), (37, 32)], fill=GRAY)
    else:
        # Folded wings
        draw.polygon([(19, 30), (17, 36), (22, 38), (22, 32)], fill=GRAY)
        draw.polygon([(38, 30), (40, 36), (35, 38), (35, 32)], fill=GRAY)
    return draw


def draw_zzz(draw, frame):
    """Draw floating ZZZ for sleeping."""
    positions = [
        (42, 16), (44, 10), (46, 4)
    ]
    for i, (x, y) in enumerate(positions):
        if frame >= i * 2:
            alpha = 255 - (frame - i * 2) * 30
            if alpha > 0:
                color = (173, 216, 230, max(0, alpha))
                draw.text((x, y - (frame - i * 2)), "Z", fill=color)
    return draw


def draw_keyboard(draw, offset_x=0, offset_y=0):
    """Draw a tiny keyboard."""
    kx, ky = 20 + offset_x, 44 + offset_y
    draw.rectangle([(kx, ky), (kx + 20, ky + 6)], fill=DARK_GRAY)
    # Keys
    for row in range(2):
        for col in range(5):
            draw.rectangle(
                [(kx + 2 + col * 4, ky + 1 + row * 3),
                 (kx + 4 + col * 4, ky + 3 + row * 3)],
                fill=WHITE
            )
    return draw


def draw_sweat(draw, frame):
    """Draw sweat drops."""
    y_offset = (frame % 3) * 2
    draw.ellipse([(40, 20 + y_offset), (42, 22 + y_offset)], fill=LIGHT_BLUE)
    if frame % 2 == 0:
        draw.ellipse([(16, 22 + y_offset), (18, 24 + y_offset)], fill=LIGHT_BLUE)
    return draw


def draw_thought_bubble(draw, frame):
    """Draw thought bubble with ? or lightbulb."""
    bx, by = 40, 8
    # Small bubbles leading to big one
    draw.ellipse([(36, 18), (38, 20)], fill=WHITE)
    draw.ellipse([(38, 14), (41, 17)], fill=WHITE)
    # Main bubble
    draw.ellipse([(bx, by), (bx + 14, by + 12)], fill=WHITE)
    draw.ellipse([(bx, by), (bx + 14, by + 12)], outline=BLACK)
    # Alternating ? and lightbulb
    if frame % 2 == 0:
        draw.text((bx + 4, by + 1), "?", fill=PURPLE)
    else:
        draw.text((bx + 3, by + 1), "!", fill=YELLOW)
    return draw


def draw_exclamation(draw, frame):
    """Draw bouncing exclamation mark."""
    y_bounce = abs((frame % 4) - 2) * 2
    draw.rectangle([(30, 6 + y_bounce), (32, 14 + y_bounce)], fill=RED)
    draw.rectangle([(30, 16 + y_bounce), (32, 18 + y_bounce)], fill=RED)
    return draw


def draw_error_x(draw, frame):
    """Draw X marks for error state."""
    y_offset = (frame % 2) * 2
    # X mark
    draw.line([(38, 8 + y_offset), (44, 14 + y_offset)], fill=RED, width=2)
    draw.line([(44, 8 + y_offset), (38, 14 + y_offset)], fill=RED, width=2)
    draw.line([(18, 10 + y_offset), (24, 16 + y_offset)], fill=RED, width=2)
    draw.line([(24, 10 + y_offset), (18, 16 + y_offset)], fill=RED, width=2)
    return draw


# ---- Sprite Sheet Generators ----

def generate_sleeping(path):
    """6 frames: bird sleeping with breathing and ZZZ."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 6, FRAME_SIZE), TRANSPARENT)
    for f in range(6):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        # Breathing: slight vertical offset
        body_oy = 1 if f in (1, 3) else 0
        draw_bird_body(draw, oy=body_oy)
        draw_crest(draw, offset_y=body_oy)
        draw_eyes(draw, blink=True)
        draw_beak(draw)
        draw_blush(draw)
        draw_wings(draw)
        draw_feet(draw)
        draw_zzz(draw, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  Generated: {path}")


def generate_waiting(path):
    """8 frames: bird looking around."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 8, FRAME_SIZE), TRANSPARENT)
    for f in range(8):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        draw_bird_body(draw)
        # Crest raised
        draw_crest(draw, offset_y=-2)
        # Look direction
        looking = "center"
        if f in (1, 2):
            looking = "left"
        elif f in (4, 5):
            looking = "right"
        blink = f == 3 or f == 7
        draw_eyes(draw, blink=blink, looking=looking)
        draw_beak(draw)
        draw_blush(draw)
        draw_wings(draw)
        draw_feet(draw)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  Generated: {path}")


def generate_working(path):
    """8 frames: bird pecking at keyboard."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 8, FRAME_SIZE), TRANSPARENT)
    for f in range(8):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        # Bobbing motion
        body_oy = 2 if f % 2 == 0 else 0
        draw_bird_body(draw, oy=body_oy)
        draw_crest(draw, offset_y=body_oy)
        draw_eyes(draw, blink=(f == 3))
        draw_beak(draw, open_beak=(f % 2 == 0))
        draw_blush(draw)
        draw_wings(draw)
        draw_feet(draw)
        draw_keyboard(draw, offset_y=body_oy)
        draw_sweat(draw, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  Generated: {path}")


def generate_thinking(path):
    """6 frames: bird looking up with thought bubble."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 6, FRAME_SIZE), TRANSPARENT)
    for f in range(6):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        draw_bird_body(draw)
        draw_crest(draw)
        draw_eyes(draw, looking="up")
        draw_beak(draw)
        draw_blush(draw)
        draw_wings(draw)
        draw_feet(draw)
        # One foot raised
        draw.line([(25, 44), (23, 46), (22, 44)], fill=ORANGE, width=1)
        draw_thought_bubble(draw, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  Generated: {path}")


def generate_notify(path):
    """6 frames: bird jumping excitedly."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 6, FRAME_SIZE), TRANSPARENT)
    for f in range(6):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        # Jumping motion
        jump_y = -4 if f in (1, 2, 3) else 0
        draw_bird_body(draw, oy=jump_y)
        draw_crest(draw, offset_y=jump_y - 2)
        draw_eyes(draw)
        draw_beak(draw, open_beak=True)
        draw_blush(draw)
        draw_wings(draw, spread=(f in (1, 2, 3)))
        draw_feet(draw, )
        draw_exclamation(draw, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  Generated: {path}")


def generate_error(path):
    """4 frames: startled bird."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 4, FRAME_SIZE), TRANSPARENT)
    for f in range(4):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        # Puffed up (slightly larger)
        draw_bird_body(draw)
        draw_crest(draw, offset_y=-3)
        draw_eyes(draw, blink=(f == 2))
        draw_beak(draw, open_beak=True)
        draw_blush(draw)
        draw_wings(draw, raised=True)
        draw_feet(draw)
        draw_error_x(draw, f)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  Generated: {path}")


def generate_stopped(path):
    """4 frames: bird settling down, closing eyes."""
    sheet = Image.new("RGBA", (FRAME_SIZE * 4, FRAME_SIZE), TRANSPARENT)
    for f in range(4):
        frame = new_frame()
        draw = ImageDraw.Draw(frame)
        draw_bird_body(draw)
        draw_crest(draw)
        blink = f >= 2
        draw_eyes(draw, blink=blink)
        draw_beak(draw)
        draw_blush(draw)
        draw_wings(draw)
        draw_feet(draw)
        sheet.paste(frame, (f * FRAME_SIZE, 0))
    sheet.save(path)
    print(f"  Generated: {path}")


def main():
    output_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), "assets", "sprites")
    os.makedirs(output_dir, exist_ok=True)

    print("Generating cockatiel sprite sheets...")

    generate_sleeping(os.path.join(output_dir, "sleeping.png"))
    generate_waiting(os.path.join(output_dir, "waiting.png"))
    generate_working(os.path.join(output_dir, "working.png"))
    generate_thinking(os.path.join(output_dir, "thinking.png"))
    generate_notify(os.path.join(output_dir, "notify.png"))
    generate_error(os.path.join(output_dir, "error.png"))
    generate_stopped(os.path.join(output_dir, "stopped.png"))

    print(f"\nAll sprites saved to: {output_dir}")


if __name__ == "__main__":
    main()
