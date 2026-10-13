# bevy-crosshair

A Bevy 0.20 plugin for configurable crosshairs and typed hit feedback.

[Playground](https://sagan-software.github.io/bevy-crosshair/) ·
[Rust API](https://sagan-software.github.io/bevy-crosshair/api/bevy_crosshair/) ·
[CS2 compatibility](docs/cs2.md) · [Implementation plan](PLAN.md) ·
[Verification](docs/verification.md) · [Playground expansion plan](docs/playground-expansion.md)

[Performance measurements and profiling](docs/performance.md) document a 57.6%
reduction in CPU time for generating 100 crosshairs with custom colors.

The playground runs the Rust plugin as WebAssembly. Change settings, supply weapon
state, and fire hits with damage metadata. An independent event observer updates
the receipt and optional hit sound. Frame measurements come from Bevy's real clock.

The configuration profile follows CS2 1.41.9.0, client 2000930. All 43 supported
settings have controls and validation. Geometry includes documented reconstructions;
actual-client pixel parity has not been established. Read the compatibility notes
before treating this as an exact CS2 renderer.

## Integration

The crate is available from this repository; it has not been published to crates.io.
Use Rust 1.99.0 or newer.

```toml
[dependencies]
bevy = "0.20.0"
bevy-crosshair = { git = "https://github.com/sagan-software/bevy-crosshair", features = ["render"] }
```

```rust
use bevy::prelude::*;
use bevy_crosshair::{Crosshair, CrosshairPlugin, Hit, HitFeedback, HitMarker, MarkerAnimation};
use std::time::Duration;

struct Damage(f32);

impl HitFeedback for Damage {
    fn marker(&self) -> HitMarker {
        HitMarker::default()
            .with_animation(MarkerAnimation::Pulse)
            .with_duration(Duration::from_millis(600))
            .scaled(1.0 + self.0 / 100.0)
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    let entity = commands.spawn(Crosshair::default()).id();
    commands.trigger(Hit { entity, metadata: Damage(40.0) });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CrosshairPlugin::<Damage>::default())
        .add_systems(Startup, setup)
        .add_observer(|hit: On<Hit<Damage>>| {
            let damage = hit.metadata.0;
            println!("Observed {damage} damage");
        })
        .run();
}
```

Install `DefaultPlugins` before `CrosshairPlugin` to enable rendering. A 2D camera
and the required UI plugins render one material per crosshair. Supply the actual
physical viewport through `CrosshairInput`; its default is 1920 × 1080.
[The playground](examples/playground/app.rs) shows window-size synchronization.

`Hit<T>` is a Bevy entity event. Your metadata only needs `HitFeedback`; it does
not need reflection or a plugin-specific wrapper. Observers can play audio, award
score, or forward analytics independently. The plugin contains no audio dependency.
Install another `CrosshairPlugin::<OtherMetadata>` to accept another metadata type.
Shared timing and rendering install once.

Later hits replace the current marker on the addressed crosshair. A zero-duration
marker clears it. Missing targets are ignored by the visual observer. Other event
observers still receive the event. Hit markers can remain visible when the normal
crosshair is disabled. Choose diagonal cross, plus, chevron, diamond, or ring
geometry with `with_shape`. `with_animation` selects linear fade, eased expansion,
eased contraction, or outward-and-return motion. The default remains linear fade.
[The feedback example](examples/feedback.rs) demonstrates these extensions without
a window.

## Configuration and host input

```rust
use bevy::color::{Alpha, LinearRgba};
use bevy_crosshair::{Crosshair, DotShape, Style};

let mut crosshair: Crosshair =
    "cl_crosshairstyle 4; cl_crosshair_length 12; cl_crosshairdot 1"
        .parse().expect("valid crosshair settings");
crosshair.set("cl_crosshaircolor_r", "255").expect("valid color channel");
let crosshair = crosshair
    .with_fill(LinearRgba::BLUE.with_alpha(0.25))
    .with_dot_shape(DotShape::Circle);
assert_eq!(crosshair.style(), Style::StaticCross);
let console_settings = crosshair.to_string();
```

Parsing and individual updates validate before changing stored settings. Export
normalizes spellings. Unknown names and unsupported legacy names produce errors.
The compatibility document defines the accepted grammar and limits.
Fill color and opacity are independent of foreground color and opacity. Center dots
can be square or circular. These visual extensions remain outside CS2 console
export; retain them separately when saving a complete application configuration.

### Custom layers

`Crosshair::with_layer` replaces the console style, fill and center dot with up
to three `Layer`s. Each layer repeats one `Mark` (dot, bar, chevron, arc or ring)
at up to four angles, measured in degrees clockwise from straight up. A layer has
its own color, an optional one-pixel outline, and an optional spread range. A
static center mark and a separate set of marks that move with spread form a
two-layer crosshair. Bars can fade toward their outer end.

```rust
use bevy::color::LinearRgba;
use bevy_crosshair::{Crosshair, Layer, Mark};

let dot = Layer::new(Mark::Dot { diameter: 2.0 }).with_outline(LinearRgba::BLACK);
let wires = Layer::new(Mark::Bar { length: 17.0, thickness: 2.0, solid: 0.3 })
    .at_angles(&[0.0, 90.0, 180.0, 270.0])
    .following_spread(4.0, 64.0);
let crosshair = Crosshair::default().with_layer(wires).with_layer(dot);
```

Layer dimensions use authored pixels and follow `cl_crosshair_screen_height`.
Visibility, recoil, and the scope, optical-sight, and grenade states still follow
the console settings. Console export excludes layers. `examples/layers.rs` prints
the generated geometry. `HitMarker::with_size`, `with_gap`, and `with_hold` set
absolute marker dimensions and an opaque hold before the animation.

`CrosshairInput` carries physical-pixel spread, shot feedback, recoil, viewport,
friendly aim, spectator state, and `WeaponState`. A weapon can be unscoped, scoped,
in a scope transition, using an optical sight, or holding one grenade. The host
owns ballistics, damage units, timers, relationships, and recoil decay.

Without the `render` feature, use `Crosshair::draw` to obtain bounded rectangles
rings, arcs and fading rectangles for another backend. Geometry generation borrows input and uses a fixed
64-primitive stack buffer. Allocation tests cover every style and empty output.

## Development

Enter the pinned environment with `nix develop`. Ordinary Cargo also works when
the compiler and platform libraries are installed.

```sh
cargo run --example basic
cargo run --example feedback
cargo run --example playground --features playground
bash scripts/build_web.sh
python3 -m http.server 8080 --directory dist
```

Native controls: Space fires, Shift selects headshots, Control selects eliminations,
Right Arrow cycles styles, and Up Arrow supplies spread.

```sh
cargo fmt --all -- --check
cargo test
cargo test --example playground --features playground
node --test scripts/presets.test.mjs
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items --all-features
cargo test --doc --all-features
nix run .#dylint
nix run .#lint-probe
nix flake check
bash scripts/coverage.sh test
bash scripts/coverage.sh bench
cargo bench --all-features --bench crosshair
```

The Dylint wrapper pins the aggregate library and matching compiler driver.
The probe requires a known ownership violation to fail and its corrected fixture
to pass. CI uses the same wrapper; a configured but inactive lint cannot satisfy it.

Test coverage requires at least 95% instrumented library Rust lines. Benchmark
execution independently requires 100%. Criterion runs the conformance cases with
`--test` under LLVM instrumentation, including an offscreen GPU pipeline and all
ten rendering modes, five marker shapes, and four marker animations. Timing runs
use the release profile without instrumentation.
Criterion also measures generation of 100 crosshairs in one iteration.
Examples, benchmark harnesses, and test harnesses are excluded from the library
coverage denominator. GPU shader instructions cannot be measured by LLVM Rust line
coverage; shader validation, pipeline creation, and browser rendering check them.

Browser FPS is a separate measurement. The display reports actual Bevy frames per
elapsed real second and the longest frame in that interval. Browser visibility,
display refresh, GPU, and host load affect the result; no universal FPS guarantee
is implied.

## Playground

The preview fits the browser window while settings scroll independently.
All 43 CS2 settings remain available.

- Browse 100 compares randomized crosshairs in one Bevy canvas. Select one to edit
  its settings. Back to gallery retains the collection and selection; Generate 100
  replaces the collection. Back to editor retains your current edits.
- Fill color has its own opacity. Dot shape switches an enabled center dot between
  square and circle geometry.
- Grenade lineup guides, sniper and optical sights, split arms, and spectators have
  explanations and preview actions. The preview identifies the simulated state;
  Reset simulation returns to ordinary aiming.
- Fire briefly opens dynamic crosshairs and supplies recoil, then restores the
  simulation values. Static shapes retain their configured gap.
- The Hits tab replays changes immediately. Choose a shape and animation, or enable
  Repeat preview. These visual previews send no gameplay event or sound. Fire sends
  the typed hit event to the independent receipt and optional sound observer.

The Background selector offers Solid and ten map scenes:

- CS2: Dust II and Overpass.
- VALORANT: Ascent and Icebox.
- Wolfenstein: Enemy Territory: Gold Rush, Fuel Dump, and Rail Gun.
- Dirty Bomb: Vault, Dockyard, and Chapel.

Images load from external sources when selected. Each scene links to its source.
Failed loads retain the solid background and show recovery text.
See [background sources](docs/backgrounds.md) for provenance.

Evaluate [scripts/check_playground.js](scripts/check_playground.js) in the loaded
page at 1280 × 800, 1280 × 400, 390 × 844, 844 × 390, and 320 × 568 CSS pixels.
It checks layout and controlled image success, failure, and stale completions.
Evaluate [scripts/check_feedback.js](scripts/check_feedback.js) on desktop and
mobile to check settings, contextual previews, event delivery, gallery navigation,
and invalid browser inputs. Both checks must report `passed: true`.

## License

MIT or Apache-2.0, at your option. See [LICENSE-MIT](LICENSE-MIT) and
[LICENSE-APACHE](LICENSE-APACHE).
