# Verification

Verified on 2026-10-10 UTC against implementation commit
[`9467fdc`](https://github.com/sagan-software/Bevy-Crosshair/commit/9467fdcefdb97177e6d1b661fd30f760591e3344).
Later documentation changes do not alter this measured code.

## Published artifact

The [Pages deployment](https://github.com/sagan-software/Bevy-Crosshair/actions/runs/38017504194)
published the [playground](https://sagan-software.github.io/Bevy-Crosshair/) and
[Rust API](https://sagan-software.github.io/Bevy-Crosshair/api/bevy_crosshair/).
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
