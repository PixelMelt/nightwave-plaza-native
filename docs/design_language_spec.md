# Nightwave Design Language Specification

This document details the visual guidelines, design tokens, and layout patterns for the **Nightwave** application. Nightwave implements a retro Windows 95/98 desktop operating system aesthetic (often referred to as the Chicago/Classic style), applied across both its **Rust (Iced) backend** desktop player and its **Web (Vue/SCSS) frontend**.

---

## 1. Visual Aesthetics & Philosophy

The design core relies on the high-fidelity recreation of the classic late 90s graphical user interface. This is not just a retro-themed skin; it is a rigid visual language with strict rules on simulated depth, typography, and color gradients.

Key principles:

- **Simulated 3D Depth**: Interfaces must look like they are carved from solid bevels. Flat elements are prohibited except for specific flat button hovers.
- **Asymmetric Borders**: Lights sources are assumed to come from the **top-left**, creating white/light highlights on top-left edges and black/gray shadows on bottom-right edges.
- **Fidelity to Era**: Inputs, windows, buttons, and scrollbars must behave exactly like Windows 98 desktop widgets.

---

## 2. Color System & Themes

Nightwave features five visual themes. The default and most prominent is the Classic Teal & Blue desktop style. All themes share a similar structure mapped to CSS variables (in SCSS) and constant colors (in Rust).

### A. Theme Palettes

| Theme Name            | Primary BG / Object Body | Light Highlight | Intermediate Shadow   | Outer Shadow | Title Bar Active Gradient                |
| :-------------------- | :----------------------- | :-------------- | :-------------------- | :----------- | :--------------------------------------- |
| **Default** (Classic) | `#c0c0c0` (Gray)         | `#ffffff`       | `#808080` (Dark Gray) | `#000000`    | `#000080` (Navy) $\rightarrow$ `#1084d0` |
| **Desert**            | `#d5ccbb` (Sand)         | `#eae6dd`       | `#a28d68` (Brown)     | `#000000`    | `#008080` (Teal) $\rightarrow$ `#00abab` |
| **Rainy**             | `#9fb0bc` (Slate)        | `#cfd7dd`       | `#6d7f8c` (Steel)     | `#000000`    | `#000080` $\rightarrow$ `#1084d0`        |
| **Rose**              | `#d4bac0` (Rose)         | `#eae0e2`       | `#a27883` (Berry)     | `#000000`    | `#4a001a` $\rightarrow$ `#910034`        |
| **High Contrast**     | `#000000` (Black)        | `#ffffff`       | `#ffffff`             | `#ffffff`    | `#000080` $\rightarrow$ `#1084d0`        |

---

## 3. Typography

Fonts must feel like pixel-perfect system fonts. Round, geometric modern web typography is strictly avoided.

- **Primary Font Family**: `Tahoma`, `Verdana`, `Segoe UI`, and fallback system sans-serif.
- **Base Font Size**: `11px` (standard desktop text size).
- **Text Alignment**: Left-aligned for text boxes; centered or left-aligned for buttons.
- **Underlines**: Underlines are reserved for hover states on links/lists or specifically mapped keyboard shortcut mnemonics (e.g., matching standard Win98 menu accelerators).

---

## 4. The 3D Bevel System

Depth is modeled by drawing double-borders where opposite corners represent light and shadow.

```
       Raised Bevel (3D Button)                 Sunken Bevel (3D Input / Track)

      [Highlight: White / Light Gray]             [Shadow: Black / Dark Gray]
       ┌───────────────────────────┐               ┌───────────────────────────┐
       │                           │               │                           │
       │        Object Body        │               │        Object Body        │
       │                           │               │                           │
       └───────────────────────────┘               └───────────────────────────┘
      [Shadow: Dark Gray / Black]                 [Highlight: Light Gray / White]
```

### Raised vs. Sunken Specifications

1. **Raised (Buttons, Window Frames, Dialogs)**:
   - Top & Left inner border: White (`#ffffff`) or Light Gray (`#dfdfdf`).
   - Bottom & Right inner border: Dark Gray (`#808080`).
   - Bottom & Right outer border: Black (`#000000`).
   - On Press / Active state: The bevel flattens. Inner highlights disappear, replaced by an inner dark-gray shadow shift, simulating mechanical depression.

2. **Sunken (Text Inputs, Option Boxes, Scrollbar Rails, Status Bar Cells)**:
   - Top & Left outer border: Black (`#000000`).
   - Top & Left inner border: Dark Gray (`#808080`).
   - Bottom & Right inner/outer border: Light Gray (`#dfdfdf`) or White (`#ffffff`).
   - Content backgrounds inside sunken fields are typically pure white (`#ffffff`) or theme-dependent light gray (`#dfdfdf` / `#c0c0c0`).

---

### Rust (Iced Desktop Client)

Since Iced does not natively support per-side border styles, the 3D bevels are created using nested containers:

- **Highlighted Outer Wrapper**: Padding on top/left with white background color.
- **Shadow Inner Wrapper**: Padding on bottom/right with dark gray or black background.
- **Container Helpers**:
  - `d3_raised`: Wraps content with white highlight and black shadow.
  - `d3_raised_window`: Wraps content with light gray highlight and black shadow.
  - `d3_sunken`: Wraps content with dark gray shadow and white highlight.
