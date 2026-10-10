# Performance

The 100-crosshair colored geometry benchmark takes 57.6% less CPU time after
replacing repeated sRGB conversion with a shared lookup table. Paired runs averaged
28.56 µs before and 12.11 µs after. The default-color gallery improved by 30.5%,
from 16.46 µs to 11.43 µs. These results measure native geometry generation.
Browser FPS and complete frame-time improvements remain unverified.

Measurements used an AMD Custom APU 0405 on 2026-10-10, Rust 1.99.0, Bevy 0.20.0,
and Criterion 0.8.2. The baseline library was revision
`aef6b666966ddb3f69edd313f7945f498ac05711`, with the new benchmarks added before
changing production code. Both binaries used the release profile, thin LTO,
one codegen unit, and no coverage instrumentation.

The gallery order was before, after, after, before. Each run used CPU 2, 50 samples,
a one-second warmup, and a three-second measurement window. Fixtures remain
outside timing. Each batch draws 100 configurations across all ten styles, with
deterministic foreground and outline byte colors, fill, and circular dot selection.
Spread changes between iterations from 0 through 40 physical pixels. Drawing generation
and return-value costs remain inside timing.

[Paired results](performance-paired.csv) record per-run means and 95% confidence
bounds in nanoseconds per iteration. The reduction is `100 * (1 - after_ns / before_ns)`;
both operands measure nanoseconds per 100-crosshair batch. Comparing the largest
optimized upper bound with the smallest baseline lower bound still gives 54.8%.

The [full suite](performance-suite.csv) retains all 74 benchmark comparisons.
Those initial runs used the benchmark defaults of 20 samples and shorter warmups.
Host load varied, especially during offscreen rendering. The marker conformance
case initially measured 3.3% slower. A repeat on CPU 2 with 50 samples measured
958.34 ns before and 985.52 ns after, a 2.8% slowdown. That case disables ordinary
crosshair geometry and does not execute the changed color conversion. These broad
timings do not establish a GPU improvement or improvements for every operation.
The paired gallery runs provide the evidence for the 50% target.

## Implementation and fidelity

Linux `perf` attributed 50.68% of baseline sampled user-space cycles to `powf`.
The optimized profile has no `powf` entry above 1% of samples. The table derives
all 256 channel values through Bevy's own `Srgba` conversion on the running target.
It retains 1,024 bytes of immutable values plus `OnceLock` synchronization state.
The first color conversion initializes it; steady-state timings exclude that
one-time cost. Alpha retains Bevy's original division by 255 without gamma correction.

The public API, settings, geometry, paint order, shaders, and marker behavior are
unchanged. Tests sweep every byte value across all eight foreground and outline
channels, compare exact float bits against Bevy, and check rejected updates and
transparency. Allocation instrumentation covers the first conversion, all ten styles,
empty output, and every marker shape and animation. Initialization and geometry
allocate no heap memory in those tests.

Test execution covers all 761 instrumented library lines, including 12 internal-test
lines. Independent benchmark execution covers all 749 production lines. No changed
production line is uncovered. LLVM branch coverage was not enabled; GPU shader
instructions are outside this denominator. Existing offscreen rendering conformance,
native and WebAssembly Clippy, public and personal lints, and Nix checks pass.

## Reproduce the measurements

Enter `nix develop`. Use the same benchmark source and compiler settings on both
library revisions. Save the baseline before applying the optimization:

```sh
cargo bench --all-features --bench crosshair -- --save-baseline before
```

After applying the optimization, compare the full suite:

```sh
cargo bench --all-features --bench crosshair -- --baseline before
```

Use longer gallery samples to check the target on another machine:

```sh
cargo bench --all-features --bench crosshair -- \
  '^gallery/' --sample-size 50 --warm-up-time 1 --measurement-time 3 \
  --baseline before
```

Criterion writes estimates and HTML reports under `target/criterion`.
Repeat comparisons in both orders under comparable host load. The saved CSV files
contain the measured results; hardware and operating conditions affect new runs.

## Profiling

`flake.nix` includes `cargo-flamegraph` and Samply on all declared platforms,
plus `perf` on Linux. The tools ran on x86_64-linux; ARM shell definitions were
evaluated but not executed. The separate `profiling` Cargo profile retains release
optimization and adds line tables without changing release benchmark settings.

When profiling on Linux, use
[cargo-flamegraph's LLD link flag](https://github.com/flamegraph-rs/flamegraph#linux):

```sh
RUSTFLAGS='-C link-arg=-Wl,--no-rosegment' \
  cargo flamegraph --profile profiling --bench crosshair --no-inline \
  -o target/colored-geometry.svg -- \
  --bench gallery/100_colored_geometry --profile-time 10
```

To profile the native playground with [Samply](https://github.com/mstange/samply),
build the example and record it:

```sh
cargo build --profile profiling --example playground --features playground
samply record --save-only -o target/playground-profile.json.gz \
  target/profiling/examples/playground
samply load target/playground-profile.json.gz
```

Before loading the profile, close the playground to finish recording.

Profiling commands use the host's existing permissions. Profiling results and
coverage-instrumented execution are separate from release timing measurements.
