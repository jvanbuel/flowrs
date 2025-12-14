# Visual Refresh Design

## Overview

A comprehensive visual overhaul for Flowrs TUI, creating a polished dark theme with modern aesthetics inspired by lipgloss/BubbleTea. Focus on one beautiful default theme rather than a theming system.

## Design Goals

- Modern, minimal, sharp aesthetic
- Clear visual hierarchy through color and spacing
- Refined selection states (no more simple color inversion)
- Tighter, more intentional layout

## Color Palette

### Primary Colors

| Role | Color | Hex | Usage |
|------|-------|-----|-------|
| Background | Deep charcoal | `#1E202A` | Main app background |
| Surface | Soft charcoal | `#323746` | Panels, alternating rows, popup backgrounds |
| Primary | Rich purple | `#8A76FF` | Default text, borders, structural elements |
| Primary muted | Soft lavender | `#9D8FD9` | Secondary text, disabled states |
| Accent | Emerald | `#00DCA0` | Selections, focus states, interactive highlights |

### Semantic Colors (Airflow States)

| State | Color | Hex |
|-------|-------|-----|
| Success | Green | `#50C878` |
| Failed | Coral red | `#FF6B6B` |
| Running | Bright cyan | `#22FFAA` |
| Queued | Muted gray | `#808080` |
| Up for retry | Amber | `#FFB347` |

Note: Success state (`#50C878`) differs from UI accent (`#00C896`) to avoid confusion between "selected" and "successful."

## Selection States & Buttons

### Button States (Popups)

| State | Background | Border | Text |
|-------|------------|--------|------|
| Default | Transparent | 1px `#7663E7` | `#7663E7` |
| Focused/Selected | `#00C89620` (emerald 12% opacity) | 2px `#00C896` | `#00C896` |
| Disabled | Transparent | 1px `#4A4A5A` | `#4A4A5A` |

Selected state combines:
- Subtle emerald background tint
- Thicker, brighter emerald border
- Text color shifts to emerald

### Table Row Selection

Note: Terminal backgrounds do not support alpha transparency. The design uses solid-color approximations.

| State | Background | Left indicator |
|-------|------------|----------------|
| Default | Terminal default (no bg set) | None |
| Alternating | `ALT_ROW_BG`: `#1E202A` | None |
| Selected | `SELECTED_BG`: `#00503C` (emerald 8% opacity approximation) | 2px solid emerald bar |
| Marked (visual selection mode) | `MARKED_BG`: `#503278` (purple 8% opacity approximation) | Small purple square |

## Spacing & Layout

### Popups
- Width: ~35% (down from 50%) for simple dialogs
- Height: Auto-sized based on content
- Padding: 1 line vertical, 2 chars horizontal inside border
- Position: Centered vertically and horizontally

### Table Layout
- Row height: 1 line with visual breathing room via alternating backgrounds
- Column padding: 1 space minimum between columns
- Header row: Subtle bottom border (`─`) in primary purple

### Headers & Title Bar
- Top bar: Solid surface color (`#24253A`) background
- Panel title: Bold text + bottom border line
- Help hint: Muted lavender (`#9D8FD9`)

### Border Styles
- Popups/modals: Rounded corners (`BorderType::Rounded`)
- Tables/panels: Sharp corners (`BorderType::Plain`)

## Implementation Approach

### Theme Module

The theme is implemented in `src/ui/theme.rs` using public constants for colors and styles:

```rust
// Color constants
pub const PURPLE: Color = Color::Rgb(138, 118, 255);     // #8A76FF
pub const ACCENT: Color = Color::Rgb(0, 220, 160);       // #00DCA0
pub const SURFACE: Color = Color::Rgb(50, 55, 70);       // #323746
pub const SELECTED_BG: Color = Color::Rgb(0, 80, 60);    // #00503C

// Style constants
pub const BUTTON_DEFAULT: Style = Style { fg: Some(TEXT_PRIMARY), bg: Some(SURFACE), ... };
pub const BUTTON_SELECTED: Style = Style { fg: Some(ACCENT), bg: Some(SELECTED_BG), ... };
pub const ALT_ROW_STYLE: Style = Style { fg: Some(TEXT_PRIMARY), bg: Some(ALT_ROW_BG), ... };
pub const SELECTED_ROW_STYLE: Style = Style { fg: None, bg: Some(SELECTED_BG), ... };
```

Replace scattered `DEFAULT_STYLE` usage with the appropriate semantic constants (`BUTTON_SELECTED`, `BUTTON_DEFAULT`, `ALT_ROW_STYLE`, `SELECTED_ROW_STYLE`, `SURFACE_STYLE`, etc.).

### Files to Modify

| File | Changes |
|------|---------|
| `src/ui/theme.rs` | Centralized color and style constants |
| `src/app/model/popup/*.rs` | Update button rendering to use `BUTTON_SELECTED`, `BUTTON_DEFAULT` |
| `src/app/model/*.rs` | Update table row rendering to use `ALT_ROW_STYLE`, `SELECTED_ROW_STYLE` |
| `src/ui.rs` | Update header bar rendering |

### Migration Strategy
1. Import theme constants from `src/ui/theme.rs`
2. Replace `DEFAULT_STYLE` with appropriate semantic constant
3. Migrate one component at a time (start with popups)

## Out of Scope

- Multiple theme support / theme switching
- Light mode
- User-configurable colors
