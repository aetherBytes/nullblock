# Hecate Avatar Sprite Specification

```
    ╔══════════════════════════════════════════════════════════════╗
    ║  HECATE - GODDESS OF MAGIC, CROSSROADS & THE NIGHT          ║
    ║  Triple-Form Pixel Art Sprite Sheet                          ║
    ╚══════════════════════════════════════════════════════════════╝
```

## Overview

Hecate is depicted in her iconic **triple-form**: three female figures standing back-to-back, each holding a torch. This sprite serves as the main avatar for the NullBlock Hecate agent interface.

---

## Canvas Specifications

| Property | Value | Notes |
|----------|-------|-------|
| **Canvas Size** | 64x64 px | Main sprite frame |
| **Character Height** | ~38-42 px | ~60-65% of canvas |
| **Sprite Sheet** | 256x128 px | 4 directions × 2 animation frames |
| **Export Format** | PNG-24 | Transparent background |
| **Color Depth** | Indexed (12 colors) | Optimized palette |

### Frame Layout (Sprite Sheet)

```
┌────────┬────────┬────────┬────────┐
│ South  │ South  │ West   │ West   │  Row 1
│ Frame1 │ Frame2 │ Frame1 │ Frame2 │  (64px)
├────────┼────────┼────────┼────────┤
│ East   │ East   │ North  │ North  │  Row 2
│ Frame1 │ Frame2 │ Frame1 │ Frame2 │  (64px)
└────────┴────────┴────────┴────────┘
   64px     64px     64px     64px
```

---

## Color Palette (12 Colors)

### Primary Colors (Body & Robes)

| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| **Void Black** | `#0d0d1a` | 13, 13, 26 | Darkest shadows, outlines |
| **Midnight Blue** | `#1a1a2e` | 26, 26, 46 | Deep robe shadows |
| **Dark Purple** | `#2d1b4e` | 45, 27, 78 | Robe mid-tones |
| **Royal Purple** | `#4a2878` | 74, 40, 120 | Robe highlights |
| **Ethereal Gray** | `#3d3d5c` | 61, 61, 92 | Robe accents, belt |

### Skin & Hair

| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| **Ghostly Pale** | `#c9bfd4` | 201, 191, 212 | Skin highlight |
| **Shadow Flesh** | `#8a7a99` | 138, 122, 153 | Skin mid-tone |
| **Raven Black** | `#1a1425` | 26, 20, 37 | Hair base |

### Magic & Fire

| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| **Torch Orange** | `#ff6b35` | 255, 107, 53 | Flame mid-tone |
| **Flame Yellow** | `#ffd93d` | 255, 217, 61 | Flame highlight/core |
| **Magic Green** | `#39ff14` | 57, 255, 20 | Aura wisps, sparkles |
| **Aura Purple** | `#8b5cf6` | 139, 92, 246 | Base aura glow |

### Aseprite Palette Import

```
GIMP Palette
Name: Hecate
#
 13  13  26	Void Black
 26  26  46	Midnight Blue
 45  27  78	Dark Purple
 74  40 120	Royal Purple
 61  61  92	Ethereal Gray
201 191 212	Ghostly Pale
138 122 153	Shadow Flesh
 26  20  37	Raven Black
255 107  53	Torch Orange
255 217  61	Flame Yellow
 57 255  20	Magic Green
139  92 246	Aura Purple
```

---

## Character Structure

### Triple-Form Layout (Top-Down View)

```
           NORTH
             │
             ▼
        ┌─────────┐
        │  Face 3 │
        │    ○    │
        └────┬────┘
             │
    ┌────────┼────────┐
    │        │        │
WEST│  ○─────┼─────○  │EAST
    │ Face1  │  Face2 │
    │        │        │
    └────────┴────────┘
             │
           SOUTH
           (Primary View)
```

### Silhouette Guide (South-Facing)

```
         Row
    ┌──────────────────────────────────────┐
  1 │            ░░░░░░░░                  │  Hair top (flowing)
  2 │          ░░████████░░                │  Hair spread
  3 │         ░██████████████░             │  Head + side faces
  4 │        ███  ████  ████  █            │  Three faces visible
  5 │         ██████████████               │  Shoulders (3 merged)
  6 │        ████  ████  ████              │  Upper torso + arms
  7 │       █████  ████  █████             │  Arms holding torches
  8 │      ▓▓████  ████  ████▓▓            │  Torches (outer)
  9 │     ░░▓▓███  ████  ███▓▓░░           │  Torch flames
 10 │        ████████████████              │  Waist + belt
 11 │       ██████████████████             │  Keys hanging
 12 │      ████████████████████            │  Robes flowing
 13 │     ██████████████████████           │  Robes mid
 14 │    ████████████████████████          │  Robes lower
 15 │   ██████████  ██████████████         │  Robes + hellhound
 16 │  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░        │  Aura base + wisps
    └──────────────────────────────────────┘

Legend: █ = Body/Robes  ▓ = Torches  ░ = Effects/Aura
```

---

## Layer Organization (Aseprite)

Create layers in this order (bottom to top):

```
📁 Hecate_Sprite
├── 🔒 Background (transparent, locked)
├── 📄 Aura_Base          ← Purple glow beneath feet
├── 📄 Aura_Wisps         ← Floating magic particles
├── 📄 Hellhound          ← Small companion dog
├── 📄 Robes_Back         ← Flowing fabric behind
├── 📄 Body_Base          ← Three torsos merged
├── 📄 Robes_Front        ← Draped fabric front
├── 📄 Belt_Keys          ← Belt with skeleton keys
├── 📄 Arms               ← Six arms (2 per form)
├── 📄 Torches_Handle     ← Torch wooden handles
├── 📄 Heads              ← Three faces
├── 📄 Hair_Base          ← Dark flowing hair
├── 📄 Hair_Flow          ← Animated hair wisps
├── 📄 Torches_Flame      ← Animated fire (ANIMATED)
├── 📄 Magic_Sparkles     ← Floating particles (ANIMATED)
└── 📄 Outline            ← Selective dark outline
```

---

## Element Details

### 1. The Torches (Primary Light Source)

Each of the three figures holds a torch. The flames provide the main lighting.

```
Torch Structure (8px tall):
     ░░
    ░██░        ← Flame tip (Flame Yellow #ffd93d)
   ░████░       ← Flame body (Torch Orange #ff6b35)
    ████        ← Flame base (Torch Orange darker)
     ██         ← Handle top (Ethereal Gray #3d3d5c)
     ██         ← Handle (Raven Black #1a1425)
     ██
     ██         ← Handle base
```

**Animation (2 frames):**
- Frame 1: Flames lean slightly left
- Frame 2: Flames lean slightly right
- Timing: 200ms per frame

### 2. The Keys (Belt Detail)

Large skeleton keys hanging from center figure's belt.

```
Key Ring (6x8px):
   ██          ← Ring
  █  █
  █  █
   ██
    █          ← Shaft
    █
   ███         ← Key teeth
```

### 3. The Dagger (Side Detail)

Curved ceremonial dagger on left figure's waist.

```
Dagger (4x7px):
  █            ← Pommel
  █            ← Handle
 ███           ← Guard
  █            ← Blade
  █
  █
  ▪            ← Tip
```

### 4. The Hellhound (Companion)

Small dark dog at the base, sitting or alert.

```
Hellhound (8x6px):
    ██         ← Ears
   ████        ← Head
  ██████       ← Body
   ████        ← Legs sitting
    ██         ← Paws
```

Color: Use Void Black (#0d0d1a) and Raven Black (#1a1425)
Eyes: Single pixel of Magic Green (#39ff14)

### 5. The Aura (Magical Effect)

Subtle glow at the base with floating wisps.

```
Aura Pattern (full width, 4px tall):
░  ░ ░   ░  ░ ░  ░   ░ ░        ← Wisps (scattered)
 ░░░░░░░░░░░░░░░░░░░░░░         ← Mid glow
  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓          ← Dense glow
   ████████████████████         ← Ground line

░ = Magic Green (#39ff14) at 50% opacity
▓ = Aura Purple (#8b5cf6) at 70% opacity
```

**Animation (2 frames):**
- Frame 1: Wisps position A
- Frame 2: Wisps shift 1-2px, some appear/disappear

---

## Directional Variations

### South (Primary - Facing Camera)
- All three faces visible
- Center figure most prominent
- Two outer figures at ~30° angles
- All three torches visible (left, center-back, right)

### West (Left Profile)
- Left figure's profile visible
- Center figure's left side visible
- Right figure mostly hidden
- Two torches visible

### East (Right Profile)
- Right figure's profile visible
- Center figure's right side visible
- Left figure mostly hidden
- Two torches visible

### North (Back View)
- Three backs visible
- Hair prominent
- Torches visible from behind (glow forward)
- Belt and keys from back

---

## Animation Frames

### Frame 1 (Base)
- Torch flames: neutral/center
- Hair: resting position
- Aura wisps: position A
- Hellhound: sitting

### Frame 2 (Animated)
- Torch flames: +1px flicker right
- Hair: +1px flow right (wind effect)
- Aura wisps: position B (shifted)
- Hellhound: same (static)

### Animation Timing
```
Frame Duration: 200ms (5 FPS)
Loop: Infinite
Ping-pong: Yes (smooth back-and-forth)
```

---

## Shading Guide

Use **2-3 levels of shading** per element:

### Robe Shading (4 levels)
```
Light source: Torches (multiple, warm)

1. Void Black (#0d0d1a)      - Deepest folds
2. Midnight Blue (#1a1a2e)   - Shadows
3. Dark Purple (#2d1b4e)     - Mid-tones
4. Royal Purple (#4a2878)    - Highlights near torches
```

### Skin Shading (2 levels)
```
1. Shadow Flesh (#8a7a99)    - Shadowed areas
2. Ghostly Pale (#c9bfd4)    - Lit areas (near flames)
```

### Torch Light Influence
- Areas near torches get warmer highlights
- Add subtle orange (#ff6b35) reflection on nearby surfaces
- Rim lighting on hair edges

---

## Export Checklist

- [ ] 64x64 individual frames (8 total)
- [ ] 256x128 sprite sheet (all frames)
- [ ] Transparent background (alpha channel)
- [ ] Indexed color mode (12 colors)
- [ ] Animation preview GIF
- [ ] Individual direction strips (64x128 each)

---

## File Naming Convention

```
hecate_avatar_64x64_sheet.png      ← Full sprite sheet
hecate_avatar_south.png            ← Individual directions
hecate_avatar_west.png
hecate_avatar_east.png
hecate_avatar_north.png
hecate_avatar_preview.gif          ← Animated preview
hecate_palette.gpl                 ← GIMP/Aseprite palette
```

---

## Quick Start (Aseprite)

1. **New File**: 64x64, Transparent background
2. **Import Palette**: Load the .gpl palette above
3. **Create Layers**: Follow layer organization
4. **Grid**: Enable 8x8 pixel grid for alignment
5. **Start with Silhouette**: Block out the triple-form shape
6. **Add Details**: Work top-to-bottom
7. **Duplicate for Directions**: Create direction variations
8. **Animate**: Add Frame 2, adjust flame/hair/wisps
9. **Export**: Sheet + individual frames

---

## Reference Images

Search terms for visual reference:
- "Hecate triple goddess statue"
- "Hecate torchbearer art"
- "Greek goddess pixel art"
- "Dark fantasy sprite RPG"
- "Torch flame pixel animation"

---

*Specification created for NullBlock Hecate Agent Avatar*
*Canvas: 64x64 | Colors: 12 | Frames: 8 (4 directions × 2 animation)*
