# Verification

Verified on 2026-10-10 UTC against implementation commit
[`9467fdc`](https://github.com/sagan-software/bevy-crosshair/commit/9467fdcefdb97177e6d1b661fd30f760591e3344).
The sections below record the initial implementation. The final section records
the later viewport, background, and naming changes. Links use the current repository
name; the initial raw samples retain the original URLs.

## Published artifact

The [Pages deployment](https://github.com/sagan-software/bevy-crosshair/actions/runs/38017504194)
published the [playground](https://sagan-software.github.io/bevy-crosshair/) and
[Rust API](https://sagan-software.github.io/bevy-crosshair/api/bevy_crosshair/).
Both pages returned HTTP 200. The published WebAssembly loaded and rendered in
foreground Chrome. The T3 preview also loads the public playground.

Browser checks covered all ten styles, setting updates, configuration export,
invalid configuration rejection, reset, scope/grenade exclusivity, and hit metadata.
A headshot carrying 83 damage produced the independent receipt
`Headshot · 83 damage · 1 events`. No application error appeared.
The optional sound observer's audio graph was checked; audible playback was not
independently assessed.

Desktop review used 1280 × 800 CSS pixels. Mobile review used 390 × 844 CSS pixels
at device pixel ratio 1.45. Controls remained usable, the preview stayed above the
settings, and neither layout overflowed horizontally. These reviews used the same
web sources as the published artifact.

## Browser frame rate

The published build ran in foreground Chrome 154.0.8037.97 on Linux, with
`document.hidden` false and document focus true. ANGLE used Vulkan on the AMD
Custom GPU 0405, RADV VANGOGH. The display runs at 60 Hz. The CSS viewport was
1247 × 660 at device pixel ratio 1; the drawing buffer was 824 × 464 pixels.

Each mode settled for one second before a 3.5-second collection period. Samples
whose endpoint fell within the first 1.1 seconds were excluded to retain complete
measurement intervals. Each mode contributed two or three samples. The values
come from actual Bevy frames divided by elapsed real time.

All ten style means ranged from 59.958 to 60.004 FPS with 40 pixels of supplied
spread. Sniper, optical sight, and flash-grenade guide means ranged from 59.985
to 60.030 FPS. The lowest individual interval was 59.892 FPS. The longest sampled
frame was 26 ms. [Raw samples](browser-frames.json) preserve each measurement.

This establishes nominal 60 FPS on the verification device. It does not establish
a strict 60 FPS minimum for every interval, a 16.67 ms maximum frame time, performance
above the display's refresh rate, or the same result on other hardware.
Startup compilation and loading are outside the steady-state sample.

T3 reported its preview panel hidden and produced unstable frame timing. A separate
blank-page animation loop reached about 121 FPS, while the Bevy preview varied
substantially. Those measurements did not establish the performance target. The
foreground Chrome measurement was made with the user's authorization.

## Tests, coverage, and benchmarks

The exact format, test, strict Clippy, rustdoc, example, and documentation-test
commands listed in the README passed in the pinned development environment.
Public Dylints passed. The activation probe rejected its known ownership violation
and accepted the corrected fixture. The applicable personal Rust lint runner passed
strict Clippy and private Dylints without candidate diagnostics. `nix flake check`
passed on x86_64-linux; other declared platforms were not executed.

The [implementation CI run](https://github.com/sagan-software/bevy-crosshair/actions/runs/38017504183)
passed every job. Its Nix checker also reported missing lint-app descriptions.
Both apps now define `meta.description`; a subsequent upstream Nix check passed
without those diagnostics.

Test execution and independent Criterion execution each reported 100% library
Rust line coverage. Benchmark execution covered 619 production lines and all 84
functions. Test execution additionally counted five internal-test lines. Five
zero-count regions remain within covered lines; this is not 100% branch coverage.
The remaining region entries map to the closing brace in `src/plugin.rs:64`,
macro fragments at `src/settings.rs:57` (columns 56, 72, and 79), and the
propagation operator at `src/settings.rs:134` (column 54). No production line
has zero hits. The required line-coverage gate passes; region coverage remains
99.47%, and branch instrumentation was not enabled.
Examples, test harnesses, and benchmark harnesses are outside the library denominator.
WESL shader instructions are outside LLVM Rust instrumentation. Shader validation,
offscreen pipeline creation, and browser rendering provide separate GPU evidence.

Allocation instrumentation checks every style and empty geometry output. Criterion
release timings on the AMD Custom APU 0405 averaged 109–235 ns for geometry and
62 ns for empty output. Offscreen Bevy update and render submission averaged
1.87–2.20 ms. [Criterion results](criterion.csv) include means and confidence bounds.
These native timings do not measure browser presentation.

## Compatibility limit

All 43 settings in the selected CS2 profile have validation and playground controls.
Exact CS2 pixel parity remains unverified. Negative-gap behavior, radial geometry,
scope shapes, grenade guides, and friendly warnings include local reconstructions.
[Compatibility notes](cs2.md) define each difference and its conformance evidence.

## Viewport and background update

The repository and Cargo package are named `bevy-crosshair`. Rust imports retain
`bevy_crosshair`, following Cargo's hyphen-to-underscore mapping.

The regression check first failed against the original page at 1280 × 400 CSS
pixels because the title, page height, preview visibility, Fire visibility, and
background options violated the requested behavior. After the fix,
`scripts/check_playground.js` passed at 1280 × 800, 1280 × 400, 390 × 844,
844 × 390, and 320 × 568. It also passed in foreground Chrome at 1247 × 660.
The checks cover container bounds, independent editor scrolling, background
success, failure fallback, credit updates, and stale success/failure completions.
All four actual publisher images loaded. Desktop and mobile visual review passed.

The local release WebAssembly build ran in foreground Chrome with document focus
and visibility confirmed. The 839 × 405 drawing buffer matched the canvas CSS
size at device pixel ratio 1. Each background loaded before a 5.5-second sample;
intervals ending within the first 1.1 seconds were excluded. Background means
ranged from 59.911 to 60.006 FPS, with a longest sampled frame of 29.3 ms.
The independent hit receipt reported `Hit · 40 damage · 1 events`.
[Raw background samples](background-frames.json) preserve the local URL and values.
These measurements establish nominal 60 FPS on this display, with the same
strict-minimum and hardware limits described above.

The library source is unchanged. LLVM line coverage continues to measure library
Rust only; it excludes the changed playground window setup and JavaScript.
The browser checks provide direct behavior evidence for those changed paths.

## Feedback and gallery expansion

The playground now includes three Enemy Territory maps and three Dirty Bomb maps,
100 randomized crosshairs in one Bevy canvas, contextual setting previews, separate
fill opacity, circular center dots, firing motion, and animated marker shapes.
The [expansion plan](playground-expansion.md) records the API and regression sequence.

Test execution reported 100% of 753 library Rust lines, including 12 internal-test
lines. Independent instrumented Criterion execution reported 100% of 741 production
lines and all 98 functions. Region coverage was 99.83% for tests and 99.56% for
Criterion. The five uncovered benchmark regions remain in `plugin.rs` and
`settings.rs`; no production line has zero hits. Branch instrumentation was not
enabled. Examples, JavaScript, test and benchmark harnesses, and GPU shader
instructions remain outside this LLVM denominator.

Allocation instrumentation passed for all crosshair styles, empty output, fill,
circular dots, and every marker shape/animation pair. Offscreen tests submitted
these extensions through the real material pipeline. Criterion times all twenty
marker combinations and a 100-crosshair geometry batch. Renderer timing uses a
three-second measurement window to collect twenty samples without extending it.
The first timing run showed regressions against earlier results on this shared
device; no performance improvement is claimed from those comparisons.

The preset generator has 100% Node line, branch, and function coverage. The browser
regression passed 99 assertions at both 1280 × 800 and 390 × 844 CSS pixels. It checks
all 43 settings, invalid configuration recovery, contextual previews, visual-preview
event isolation, actual hit metadata, 100 rendered gallery entities, selection,
return navigation, regeneration, and invalid WebAssembly boundary values.
Layout and controlled image-request checks passed at 1280 × 800, 1280 × 400,
390 × 844, 844 × 390, and 320 × 568. Keyboard selection opens Style with focus;
reduced-motion preference stops repetition and selects the stationary fade.

Browser behavior checks do not establish instrumented coverage of the JavaScript
or WebAssembly example branches. The strict WebAssembly Clippy check retains one
explicit `clippy::too_many_arguments` expectation on the existing nine-scalar
`simulate` export to preserve its signature. Public and personal native Dylint
checks retain their full all-target/all-feature scope.

Logs, coverage output, Criterion reports, screenshots, and frame samples are kept
under `/var/mnt/nixsd/agent-storage/artifacts/bevy-crosshair` and the SD-backed
project target. Builds use `/tmp/deck-heavy-build.lock`; byte and inode checks run
before large jobs and every sixty seconds during them.

The final local release ran in focused, visible Chrome 154.0.8037.97 through ANGLE
Vulkan on the AMD Custom GPU 0405, RADV VANGOGH. At 1280 × 800 CSS pixels and device
pixel ratio 1, the editor canvas was 872 × 529 and the gallery canvas was
1217 × 600. Each mode settled for 2.5 seconds before a twelve-second collection;
intervals ending within the collection's first 1.5 seconds were excluded.
The editor repeated an animated diamond marker over Gold Rush; the gallery rendered
100 actual crosshair entities over that background.

Eleven editor intervals averaged 60.004 FPS, with a 59.946 FPS minimum interval and
19.0 ms longest frame. Ten gallery intervals averaged 60.002 FPS, with a 59.958 FPS
minimum interval and 23.3 ms longest frame. This establishes nominal 60 FPS on this
60 Hz display, subject to the strict-minimum and hardware limits stated above.
An earlier gallery sample at the same size averaged 48.688 FPS. CPU profiling and
repeat measurements were completed; no renderer optimization was needed to reach
the target in the final sample. These samples demonstrate sensitivity to host
conditions rather than a guaranteed minimum frame rate.

Pixel checks isolated gap changes by disabling recoil translation. Dynamic cross
expanded from 32 × 32 to 50 × 50 pixels after Fire and returned to 32 × 32.
Shot-feedback cross expanded from 32 × 32 to 52 × 52 and returned to 32 × 32.
Static cross remained 32 × 32 throughout. The separate host-motion test verifies
recoil, restart behavior, and exact restoration of the configured baseline.
