---
version: 1
slug: "web-index-html"
primary_target: "web/index.html"
related_targets: ["web/style.css","web/app.js"]
---

# Crosshair playground

Mode: Operate. Bevy developers adjust a crosshair and trigger hits while observing
its behavior in the running renderer. All settings remain accessible.

## Direction contract

THESIS: Keep the rendered crosshair beside its controls, with immediate feedback.
OWN-WORLD: A pale blue control surface, dark blue preview, dark ink labels, and
blue focus states. Use system sans text and tabular numeric values.
STORY: Choose a style, change geometry, then simulate movement, recoil, and hits.
FIRST VIEWPORT: A compact header, a large preview on the left, and a 360-pixel
settings panel on the right. Put the Fire button below the preview. On mobile,
split the available height between the preview and an independently scrolling editor.
Use a side-by-side layout in short landscape windows. Keep the canvas inside its
container. Show measured FPS beside the preview. Put a background selector above
the canvas with publisher credits, loading state, and a solid fallback.
FORM: A settings editor chosen under the user's delegated design defaults.
The signature interaction is the same typed hit updating both a visible marker
and an independent event receipt with optional sound. No concept-seed is needed
for the user's prescribed task-focused application layout.
FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
