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
blue editor focus states. Gallery selection and focus use white outlines over
the dark renderer. Use system sans text and tabular numeric values.
STORY: Choose a style, tune geometry or fill, compare live crosshairs, then
simulate spread, recoil, scope, grenade, spectator, and hit feedback. Contextual
help and preview actions cover split arms, scopes, grenade guides, friendly
warnings, and spectator views. Hit markers offer five shapes, four animations,
replay, and repeat preview.
FIRST VIEWPORT: A compact header, a large preview on the left, and a 360-pixel
settings panel on the right. Put the Fire button below the preview. At widths up
to 760px, default to a 3:2 split between preview and independently scrolling
editor. At heights up to 500px and widths from 600px, use the short landscape
two-column layout. Keep the canvas
inside its container. Show measured FPS beside the preview. Put the Background
selector above the canvas with a Solid default, selected source credit, loading
state, and unavailable-image state. Gallery mode fills the preview pane and hides
the editor and preview actions. Show 100 actual crosshairs rendered by Bevy in a
desktop grid with 10 columns or a mobile grid with 5 columns. The preview
viewport scrolls internally around a render stage that is at least 600px high
on desktop and 1200px high at mobile widths.
FORM: A settings editor chosen under the user's delegated design defaults.
Keep the four existing native tabs in order: Crosshair, Hits, Simulation, Config.
The signature interaction is the same typed hit updating both a visible marker
and an independent event receipt with optional sound. No concept-seed is needed
for the user's prescribed task-focused application layout.
SCENE SOURCES: Solid is the default. All ten scene images remain externally
hosted, and no scene raster is committed. The selector shows the selected credit
link.

- Valve credits CS2 Dust II and Overpass ([Counter-Strike 2](https://www.counter-strike.net/cs2)).
- Riot Games credits VALORANT Ascent and Icebox ([VALORANT maps](https://playvalorant.com/en-us/maps/)).
- Mark Fry credits Enemy Territory Gold Rush, Fuel Dump, and Rail Gun ([Wolfenstein: Enemy Territory](https://www.markfry.tv/level-design/wolfenstein-enemy-territory/)).
- MMO Culture credits Dirty Bomb Vault ([article](https://mmoculture.com/2017/07/dirty-bomb-first-new-mercenary-added-after-separating-from-nexon/)).
- Geek Blender credits Dirty Bomb Dockyard ([article](https://geekstuffpod.wixsite.com/geek-blender/single-post/2017/05/26/dirty-bomb-ya-tiene-nuevo-mapa-dockyard)).
- PC Gamer credits Dirty Bomb Chapel ([review](https://www.pcgamer.com/dirty-bomb-review-beta/)).
FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
