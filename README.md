# Bevy Crosshair

A Bevy 0.20 plugin for configurable crosshairs and typed hit feedback.

[Playground](https://sagan-software.github.io/Bevy-Crosshair/) ·
[Rust API](https://sagan-software.github.io/Bevy-Crosshair/api/bevy_crosshair/) ·
[CS2 compatibility](docs/cs2.md) · [Implementation plan](PLAN.md)

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
bevy_crosshair = { git = "https://github.com/sagan-software/Bevy-Crosshair", features = ["render"] }
```

```rust
use bevy::prelude::*;
use bevy_crosshair::{Crosshair, CrosshairPlugin, Hit, HitFeedback, HitMarker};

struct Damage(f32);

impl HitFeedback for Damage {
    fn marker(&self) -> HitMarker {
        HitMarker::default().scaled(1.0 + self.0 / 100.0)
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
crosshair is disabled.

## Configuration and host input

```rust
use bevy_crosshair::{Crosshair, Style};

let mut crosshair: Crosshair =
    "cl_crosshairstyle 4; cl_crosshair_length 12; cl_crosshairdot 1".parse()?;
crosshair.set("cl_crosshaircolor_r", "255")?;
assert_eq!(crosshair.style(), Style::StaticCross);
let console_settings = crosshair.to_string();
# Ok::<(), bevy_crosshair::SettingError>(())
```

Parsing and individual updates validate before changing stored settings. Export
normalizes spellings. Unknown names and unsupported legacy names produce errors.
The compatibility document defines the accepted grammar and limits.

`CrosshairInput` carries physical-pixel spread, shot feedback, recoil, viewport,
friendly aim, spectator state, and `WeaponState`. A weapon can be unscoped, scoped,
in a scope transition, using an optical sight, or holding one grenade. The host
owns ballistics, damage units, timers, relationships, and recoil decay.

Without the `render` feature, use `Crosshair::draw` to obtain bounded rectangles
and rings for another backend. Geometry generation borrows input and uses a fixed
64-primitive stack buffer. Allocation tests cover every style and empty output.

## Development

Enter the pinned environment with `nix develop`. Ordinary Cargo also works when
the compiler and platform libraries are installed.

```sh
cargo run --example basic
cargo run --example playground --features playground
bash scripts/build_web.sh
python3 -m http.server 8080 --directory dist
```

Native controls: Space fires, Shift selects headshots, Control selects eliminations,
Right Arrow cycles styles, and Up Arrow supplies spread.

```sh
cargo fmt --all -- --check
cargo test
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
ten rendering modes. Timing runs use the release profile without instrumentation.
Examples, benchmark harnesses, and test harnesses are excluded from the library
coverage denominator. GPU shader instructions cannot be measured by LLVM Rust line
coverage; shader validation, pipeline creation, and browser rendering check them.

Browser FPS is a separate measurement. The display reports actual Bevy frames per
elapsed real second and the longest frame in that interval. Browser visibility,
display refresh, GPU, and host load affect the result; no universal FPS guarantee
is implied.

## License

MIT or Apache-2.0, at your option. See [LICENSE-MIT](LICENSE-MIT) and
[LICENSE-APACHE](LICENSE-APACHE).
