# Playground expansion

## Intended behavior

- Add at least three named maps each for Wolfenstein: Enemy Territory and Dirty Bomb.
  Verify their images, retain source credits, and keep the existing loading recovery.
- Explain every contextual setting without removing options. Place grenade, scope,
  split, and spectator preview actions beside their settings. Identify the active
  simulation and provide a return to ordinary aiming.
- Render 100 randomized crosshairs in one Bevy canvas. Opening one retains its
  settings; returning retains the collection and selection. Regeneration is explicit.
- Add independently colored, independently translucent crosshair fill and a circular
  center-dot option through the reusable Rust API.
- Fire supplies transient spread, shot feedback, and recoil, then returns to the
  configured simulation values. Static shapes retain their static gap behavior.
- Hit-setting edits replay a marker without emitting gameplay hits or sound.
  Provide repeat preview, shape choices, and eased expand, contract, and pulse motion.
- Deploy and verify the published playground, including foreground frame measurements.

## Invariants and API

Keep all existing public signatures and the exact CS2 console text profile.
`Crosshair::with_fill(LinearRgba)` and `with_dot_shape(DotShape)` add visual
extensions. Console export excludes these extensions and the editor says so.
Fill is painted behind the crosshair with its own alpha. Circular styles use a
disk; the square style uses a square. Zero interior radius or side length has no fill.
Dynamic quad includes its arc-length offset in the interior radius. Center dots support
the closed `Square` and `Circle` variants.

`HitMarker::with_shape(MarkerShape)` and `with_animation(MarkerAnimation)` retain
the existing lifetime, scaling, and metadata event contract. Shapes are `Cross`,
`Plus`, `Chevron`, `Diamond`, and `Ring`. Animations are `Fade`, `Expand`,
`Contract`, and `Pulse`. The default remains the existing diagonal linear fade.
Animated modes use normalized lifetime and cubic easing. Zero duration is invisible;
expired markers cannot revive without a new event or explicit preview.

Reject malformed browser inputs before queuing changes. Keep fill colors finite,
clamp alpha to zero through one, and preserve bounded geometry. Gallery input is
limited to 100 entries. Each frame borrows configuration and uses constant stack
storage per crosshair. Gallery rendering is O(n), with n bounded at 100; retained
materials are O(n). No frame builds owned configuration strings or meshes.

Preview actions change visible host state, not console policy. Marker previews
replace the visual component directly; Fire triggers the shared typed hit event.
Grenade guides align screen landmarks and do not predict trajectories. Scope and
grenade states remain mutually exclusive. Spectator previews supply a distinct
observed configuration so ownership policy is visible.

## Checkpoints and acceptance gates

1. Add an external regression and example, then record the missing-API failure.
2. Define documented types and signatures, then compile the guide and rustdoc.
3. Implement bounded geometry and animation; test lifetime and geometry boundaries.
4. Implement browser previews and the gallery through the actual Bevy renderer.
5. Cover new library paths in tests and instrumented Criterion execution. Check
   zero allocations, shader validation, and offscreen rendering.
6. Run `cargo fmt --all -- --check`, `cargo test`, strict all-target/all-feature
   Clippy, private rustdoc, both coverage scripts, Criterion, public Dylints,
   the activation probe, the applicable personal lint runner, and Nix checks.
7. Verify all background loads; stale loads and failures; contextual previews;
   gallery selection, return, regeneration, and keyboard access; hit previews,
   real hit receipts, reduced motion, desktop, mobile, and short windows.
8. Review rendered desktop and mobile views. Finish with standalone documentation,
   example and documentation tests, then publish and verify CI and Pages.

Use the SD-backed target, temporary, cache, and artifact directories. Check bytes
and inodes before large jobs and every 60 seconds during them. Reserve 50 GiB or
10% of SD capacity, whichever is larger, plus estimated output. Retain 5 GiB on
`/home`. Serialize builds with `/tmp/deck-heavy-build.lock`; do not delete outputs.

## Execution record

The external extension tests initially failed because `DotShape`, `MarkerShape`,
`MarkerAnimation`, and the fill and shape builders did not exist. The runnable
feedback guide and public API then established the intended calls before the
implementation was completed. A separate event regression reproduced a marker
remaining transparent after its first frame. The clock now advances according to
remaining lifetime, independently of opacity.

The implementation preserves the CS2 console profile and existing public
signatures. Focused modules own dot shape, marker shape, and marker animation.
Shared public conformance cases run both as tests and under instrumented Criterion.
Allocation checks cover fill, circular dots, and every shape/animation pair.

The browser regression reproduced a gallery selection leaving the Hits tab open.
Selection now opens the Crosshair tab at Style and moves keyboard focus there.
The browser checks retain all 43 controls and cover direct previews, event
isolation, selection, return navigation, and invalid boundary inputs.

Builds and validation use the shared heavy-job lock and SD-backed output paths.
No cleanup was performed. Final measured results and verification limits are
recorded in [verification](verification.md).
