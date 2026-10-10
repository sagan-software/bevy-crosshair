# Bevy Crosshair implementation plan

## Intent

Initialize `sagan-software/Bevy-Crosshair` as a small Rust plugin for Bevy 0.20.
Start with a runnable integration example and its public API. Define the types,
methods, rustdoc examples, and intentional implementation comments before filling
in behavior. Publish an interactive GitHub Pages playground that uses the actual
plugin, exposes the settings, and measures its frame rate. Target at least 60 FPS
on the verification device, while allowing the browser's display refresh rate to
set a higher ceiling. Do not present a device-specific measurement as a universal
guarantee.

Copy the current Counter-Strike 2 crosshair settings and their documented
semantics. Extend them with metadata-driven hit markers. Let independent Bevy
systems observe hit metadata for sounds, scoring, or other responses. Keep game
damage units and audio assets in the host application. Prefer standard Rust and
Bevy traits over new wrappers and application-specific event buses.

Use `sagan-software/template-bevy` as tooling guidance, including a pinned Nix
environment and working Dylints. Preserve ordinary Cargo use. Require at least
95% measured production line coverage, aim to exercise every reachable new branch,
and provide Criterion benchmarks. The user explicitly requires 100% instrumented
production Rust line coverage from benchmark execution. Measure that separately
from test coverage, benchmark timings, and browser frame rate.

## Source and compatibility research

- Bevy 0.20.0 was released on 2026-10-08. Its manifest requires Rust 1.97.1.
  Use Rust 1.99.0, matching the inspected template's current toolchain.
  Sources: <https://bevy.org/news/bevy-0-20/> and
  <https://github.com/bevyengine/bevy/blob/v0.20.0/Cargo.toml>.
- CS2 build 1.41.9.0, client 2000930, dated 2026-10-08, has ten styles, new
  `cl_crosshair_*` size settings, RGBA crosshair and outline colors, full and half
  outlines, and an authored screen-height setting. Inspect the shipped convars,
  settings layout, and settings script captured by GameTracking-CS2.
  Source: <https://github.com/SteamDatabase/GameTracking-CS2>.
- The Valve developer wiki returned HTTP 403. Captured client declarations are
  evidence for names, limits, defaults, and visible control applicability.
  They do not prove undocumented client geometry or weapon simulation formulas.
  Record those verification limits beside the compatibility matrix. Do not claim
  pixel-identical CS2 parity without matching captures from the actual client.
- The inspected template metadata points to obsolete `lints/*` paths. The current
  public Dylints repository uses `crates/*`. Pin one revision consistently and
  verify the selected libraries and a failing lint probe.

## Invariants and API direction

- Crosshair styles, outline modes, scope states, grenade kinds, and spectator
  policies are closed enums. Time spans use `Duration`; positions use `Vec2`.
- Validate configuration at ingress. Reject non-finite numbers, unknown settings,
  unsupported legacy settings, invalid enum values, and values outside each
  captured command's bounds. Failed updates leave the previous value intact.
- Keep one authoritative crosshair configuration per entity. Derive geometry,
  scale, opacity, visibility, and hit-marker lifetime from configuration and input.
- Supply weapon spread, shot feedback, recoil offset, scope state, grenade hold
  duration, friendly aim, and spectator state explicitly. The plugin does not
  infer game physics or friendship from rendering data.
- Use a typed Bevy hit event carrying its target crosshair and user-owned metadata.
  A small trait maps that metadata to a marker. Other observers receive the same
  event independently. No audio dependency belongs in the reusable plugin.
- Retain only the newest marker per crosshair. Later hits replace earlier markers;
  zero duration clears a marker. Despawned or unrelated targets do not panic.
- Geometry generation borrows settings, takes constant bounded space, and performs
  work proportional to the bounded primitive count. Avoid per-frame allocation in
  geometry and stable-state updates. Measure allocation behavior before claiming it.
- Rendering uses an optional Bevy backend. Headless tests exercise configuration,
  geometry, event delivery, marker replacement, timing, and visibility policies.
  The playground renders the Rust plugin compiled to WebAssembly.

## Implementation checkpoints

1. Write the public runnable example and integration tests. Run a focused test and
   record the genuine missing-API failure before implementing production behavior.
2. Create focused modules with module documentation and a narrow `lib.rs` facade.
   Define public types, signatures, and documented usage. Compile the example,
   build private and public rustdoc, and run documentation tests.
3. Add intentional comments to nontrivial validation, rendering, ordering,
   event-delivery, and failure-handling stages before filling in those stages.
4. Implement through the example and public integration tests, one coherent slice
   at a time. Add a separate internal test module for a private invariant after
   the external tests pass.
5. Add a native example and a browser playground with all current settings,
   hit metadata controls, independent event feedback, optional synthesized hit
   sound, style selection, reset, and configuration import/export. Measure real Bevy frame
   timing rather than an unrelated JavaScript animation loop.
6. Add Criterion cases for each hot operation and every style, static and moving
   input, empty output, active/expired markers, and one rendered entity, the usual HUD workload.
   Run benchmarks in a stable release configuration and preserve measured results.
7. Measure coverage, close uncovered production branches, and run every gate below.
   Update README and standalone documentation in a separate final prose-only edit.
   Run examples and rustdoc tests last. If code changes again, repeat this final
   documentation checkpoint.
8. Create the requested GitHub repository with the configured user identity.
   Publish CI, enable Pages, deploy the playground and API documentation, then
   verify the deployed URL, controls, rendering, errors, desktop/mobile layouts,
   and frame timing. Record the exact deployed commit and verification limits.

## Acceptance checklist

- [x] Public example and integration tests precede production behavior.
- [x] Missing-API failure, signature compilation, and implementation stages recorded.
- [x] Current CS2 setting matrix maps every setting to behavior and tests.
- [x] Every accepted value has defined units and every rejected legacy setting is documented.
- [x] Generic hit metadata reaches both marker rendering and independent observers.
- [x] Replacement, expiry, despawn, disabled output, and target isolation are tested.
- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo test` passes.
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [x] `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items --all-features` passes.
- [x] `cargo test --doc --all-features` passes.
- [x] `nix run .#dylint` runs all selected public Dylints without diagnostics.
- [x] `nix run .#lint-probe` proves that a known violation fails and its fix passes.
- [x] The applicable Liamc lint command finishes without candidate diagnostics.
- [x] `nix flake check` passes.
- [x] Measured production line coverage is at least 95%; exact remaining gaps recorded.
- [x] Every hot operation and style has a Criterion case, and the suite executes.
- [x] Instrumented benchmark execution covers 100% of production Rust lines.
- [x] Allocation instrumentation verifies any zero-allocation claim.
- [x] WebAssembly build uses Bevy 0.20 and the reusable plugin.
- [x] Desktop/mobile visual review and interaction checks pass.
- [x] The deployed playground has a recorded frame-rate sample and environment.
- [x] GitHub Pages serves the tested commit with working asset paths.
- [x] Commit author, committer, message, and trailers contain only user attribution.
- [ ] Actual CS2 captures establish exact visual parity. Reconstruction differences remain documented in `docs/cs2.md`.

## Verification record

The first `cargo test --test api` failed with six E0432 unresolved-import errors
for `Crosshair`, `CrosshairPlugin`, `Hit`, `HitFeedback`, `HitMarker`, and `Style`.
The public example and tests existed before their production definitions.
The initial direct invocation first exposed a missing C linker; the pinned Nix
environment supplies it. Signature-only modules compiled, and their documentation built before behavior
was implemented. Intent comments preceded implementation. Public tests then passed;
a separate internal marker test was added after that external checkpoint.

The public repository exists at <https://github.com/sagan-software/Bevy-Crosshair>.
The implementation was pushed as `9467fdcefdb97177e6d1b661fd30f760591e3344`.
Its author and committer are Bill Curry, using the configured Git identity.

The template was inspected at `1f515985e4bd4cf63f0f52f92e3bb5b6353dbe13`.
The selected current Dylints revision is
`03fc5f93c2bcdd02a8f7f2fbe2710a315d5e06a9`.
Its aggregate is a dynamic library, so the Nix wrapper uses its pinned Dylint
driver and library directly. The older template used a different packaged runner.

The complete native Criterion timing run passed on an AMD Custom APU 0405.
Geometry means ranged from 109 to 235 ns across the ten styles and two spread
inputs. Empty geometry averaged 62 ns. Offscreen Bevy update and render submission
averaged 1.87–2.20 ms across styles. These timings do not measure browser FPS.

LLVM reports 100% line coverage for test execution and independently for Criterion
execution. Benchmark coverage includes 619 library lines and 84 functions. Test
coverage also includes the five lines of the internal test. WESL shader execution
is outside LLVM Rust instrumentation. Five zero-count regions remain within
otherwise covered lines; line coverage is not a branch-coverage claim.

The installed personal-lint skill refers to a missing `scripts/nix-safe` dispatcher.
The inspected local checkout supplies a Rust runner instead. Its current Nix app
ran strict Clippy and all bundled private Dylints with `--all-targets --all-features`
and no candidate diagnostics. The public pinned Dylint aggregate and failing/fixed
probe passed separately. The missing older multi-ecosystem dispatcher was not
silently treated as a passing gate.

The initial T3 browser measurements were throttled and did not establish 60 FPS.
A blank-page animation loop reached about 121 FPS. Bevy frame callbacks used about
10.7 ms on average in one diagnostic sample while the preview delivered about
8 frames per second. T3 continued reporting the panel hidden after the user opened
it. The user authorized foreground Chrome verification. The published artifact
then averaged 59.96–60.03 FPS across ten styles and three additional render modes
on the 60 Hz verification display. The longest sampled frame was 26 ms. These
measurements establish nominal 60 FPS on that device, not a strict lower bound
for every interval or frame. See [the verification record](docs/verification.md)
for the environment, raw samples, deployed artifact, and remaining limits.
