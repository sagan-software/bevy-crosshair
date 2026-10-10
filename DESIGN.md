---
name: Bevy Crosshair
description: Live crosshair preview and settings editor for Bevy developers.
colors:
  primary: "#175fc3"
  primary-deep: "#114eaa"
  focus: "#3880dd"
  canvas: "#e9eff5"
  panel: "#f5f8fb"
  panel-selected: "#e1ecf9"
  ink: "#182c41"
  muted: "#52667a"
  line: "#c8d4e0"
  control-border: "#b3c2d1"
  preview: "#1a2633"
  surface: "white"
  button-surface: "#f8fafc"
  button-hover: "#dae6f3"
  error: "#91282d"
  error-surface: "#fde7e6"
typography:
  title:
    fontFamily: "ui-sans-serif, system-ui, sans-serif"
    fontSize: "20px"
    fontWeight: 650
    letterSpacing: "-0.025em"
  body:
    fontFamily: "ui-sans-serif, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
  navigation:
    fontFamily: "ui-sans-serif, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 400
  measurement:
    fontFamily: "ui-sans-serif, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
  configuration:
    fontFamily: "ui-monospace, monospace"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.65
rounded:
  sm: "4px"
  md: "5px"
  lg: "6px"
spacing:
  xs: "8px"
  sm: "12px"
  md: "16px"
  lg: "20px"
  xl: "24px"
  page: "28px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "white"
    rounded: "{rounded.md}"
    padding: "8px 28px"
  button-neutral:
    backgroundColor: "{colors.button-surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    padding: "8px 12px"
  settings-tab:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.muted}"
    rounded: "4px 4px 0 0"
    padding: "12px 5px"
  settings-tab-selected:
    backgroundColor: "{colors.panel-selected}"
    textColor: "{colors.primary-deep}"
    rounded: "4px 4px 0 0"
    padding: "12px 5px"
  select:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.sm}"
    padding: "8px"
---

# Design System: Bevy Crosshair

## Overview

Design focus: Crosshair settings editor.

Crosshair settings stay beside their renderer. A dark preview, pale editor, and nearby measured feedback keep adjustment and observation together.

Key characteristics:

- Pale blue-gray application surfaces frame a dark slate renderer.
- Blue marks interactive and selected states.
- System sans text carries the interface; configuration text uses monospace.
- The settings editor groups four categories behind tabs.

## Colors

Pale blue-gray surfaces, dark ink, and one blue interaction family frame the renderer and its user-controlled crosshair.

### Primary

- Clear blue is used for links, primary actions, range controls, and active-tab indicators.
- Deep blue is used for the primary action's hover state and selected-tab text.
- Focus blue marks keyboard focus with an offset outline.

### Neutral

- Canvas and panel separate the page background from the settings editor.
- Ink and muted text carry labels, values, and supporting status.
- Lines and control borders separate the editor and preview from their surroundings and outline form controls.
- Preview slate gives the rendered crosshair a dark field.
- Selected panel gives the active settings tab a pale blue fill.

### Feedback

- Error red on pale red is reserved for the renderer error message.

Blue identifies interaction and selection in the editor. The configurable crosshair color remains renderer content.

## Typography

The interface uses one system sans family. The configuration editor switches to a monospace family, and measurement text uses tabular numerals.

### Hierarchy

- Title: Compact, semibold application heading; it steps down on narrow screens.
- Body: Default size for field labels and general interface text.
- Navigation: Smaller text for the header links.
- Measurement: Compact values for FPS and frame time; the FPS value is emphasized.
- Configuration: Monospace text with generous line spacing for console settings.

Keep FPS, frame time, and hit receipt values tabular so changing digits do not shift their alignment.

## Layout

The desktop header is 68px tall. The main area is capped at 1800px and divides into a flexible preview with a 360px settings editor. The preview pane uses the larger page spacing token for its padding.

At widths up to 760px, the header shortens and the layout becomes a single column. The preview sticks to the top while scrolling; its viewport is 30dvh tall with a 150px minimum. The editor follows below it, and its tabs scroll with the settings. On desktop, the editor scrolls independently and its tabs stick to the editor top.

The spacing rhythm comes from the compact steps in the frontmatter. The page uses a wider desktop gutter and tighter mobile preview padding.

Keep the preview above the editor on narrow screens and sticky while settings scroll beneath it.

## Elevation & Depth

The page separates the renderer, editor, and outer canvas through tonal contrast and thin borders. It uses no broad drop shadows. The selected tab's inset bottom rule marks state without lifting the surface.

## Shapes

Buttons use softly rounded corners. Selects and form fields use tighter corners, while the renderer viewport uses the broadest radius. Thin borders define the editor edge and field boundaries.

## Components

### Buttons

The primary Fire button uses the blue accent and white text. Neutral controls use a light surface and dark ink. Hover changes the fill; keyboard focus gets a visible offset outline. Disabled actions are dimmed and show a waiting cursor.

### Inputs and fields

Range rows place the label and value above a full-width slider. Selects span the row. Checkboxes keep their native control shape with the blue accent. Color inputs show a compact swatch. Advanced settings use disclosure rows separated by top borders.

### Navigation

The header links remain understated. Four equal settings tabs keep the order Crosshair, Hits, Simulation, Config. The selected tab uses a pale blue fill, deeper blue text, and an inset blue bottom rule. Tabs stick while the desktop editor scrolls and become static above the settings on mobile.

### Preview and feedback

The preview bar pairs the current crosshair style with live FPS and maximum frame time. Fire and hit-sound controls sit below the renderer with the event receipt aligned to the opposite edge. Renderer errors appear as red text on a pale red surface.

### Configuration

The configuration editor uses a monospace textarea. Apply, Copy, and Reset defaults sit together beneath it.

## Do's and Don'ts

### Do:

- Do keep the live renderer and its controls immediately available.
- Do keep the preview sticky above the settings editor on narrow screens.
- Do preserve the four settings tabs and their current order.
- Do use tabular numerals for FPS, frame time, and hit receipt values.
- Do keep configuration text in a monospace field.

### Don't:

- Don't use the configurable crosshair color as the editor's interaction accent.
- Don't fix the rendered crosshair to the editor's blue palette.
