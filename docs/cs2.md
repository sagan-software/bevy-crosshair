# CS2 compatibility

This profile targets CS2 1.41.9.0, client 2000930, released on 2026-10-08.
Names, defaults, ranges, and control applicability come from the captured client
[convars](https://github.com/SteamDatabase/GameTracking-CS2/blob/f68ba6aa2473367589e7cb249fc38814c9b90ff0/DumpSource2/convars.txt),
[settings layout](https://github.com/SteamDatabase/GameTracking-CS2/blob/f68ba6aa2473367589e7cb249fc38814c9b90ff0/game/csgo/pak01_dir/panorama/layout/settings/settings_crosshair.xml), and
[settings script](https://github.com/SteamDatabase/GameTracking-CS2/blob/f68ba6aa2473367589e7cb249fc38814c9b90ff0/game/csgo/pak01_dir/panorama/scripts/settingsmenu_crosshair.js).

These files specify configuration, but they do not specify every CPU-side geometry
formula. Actual-client pixel comparisons remain necessary before claiming exact
visual parity. The reconstruction choices below are part of this implementation,
not verified Valve geometry formulas.

## Current crosshair controls

- `crosshair`: enabled, default true.
- `cl_crosshairstyle`: integer 0 through 9, default 7. Values identify dynamic
  cross, dynamic circle, legacy dynamic cross, static circle, static cross,
  shot-feedback cross, dot, dynamic quad, static square, and static quad.
- `cl_crosshair_length`: 0 through 255 authored pixels, default 8.
- `cl_crosshair_thickness`: 0 through 32 authored pixels, default 2. Rendering
  retains a minimum thickness of one physical pixel.
- `cl_crosshair_gap`: -3840 through 3840 authored pixels, default 4. The menu
  exposes 0 through 128, or -10 through 128 for the legacy dynamic cross.
- `cl_crosshair_screen_height`: at least 240 pixels, default 1080. Scale authored
  dimensions by current physical viewport height divided by authored height.
- `cl_crosshair_drawoutline`: none 0, full 1, half 2; default full.
- `cl_crosshaircolor_r`, `_g`, `_b`, `_a`: integer channels 0 through 255,
  defaults 0, 255, 0, 255.
- `cl_crosshairoutline_r`, `_g`, `_b`, `_a`: integer channels 0 through 255,
  defaults 0, 0, 0, 255.
- `cl_crosshairdot`: center dot, default false.
- `cl_crosshair_t`: hide the upper bar, default false.
- `cl_crosshair_recoil`: follow supplied predicted recoil, default true.
- `cl_crosshair_dynamic_spread_limit`: 0 through 255, default 255. The client
  describes this as additional spread beyond a baseline of 128 pixels.
- `cl_crosshair_dynamic_splitdist`: 0 through 127, default 3. Only the legacy
  dynamic cross uses this split threshold.
- `cl_crosshair_dynamic_maxdist_splitratio`: 0 through 1, default 1. The legacy
  dynamic cross divides bar length into inner length times one minus this ratio
  and outer length times this ratio. Static quad also exposes this control.
- `cl_crosshair_dynamic_splitalpha_innermod`: 0 through 1, default 0.
- `cl_crosshair_dynamic_splitalpha_outermod`: 0.3 through 1, default 1.
- `cl_crosshair_friendly_warning`: enabled 1 or disabled 0, default 1.

## Scope, grenade, and spectator controls

- `cl_crosshair_sniper_width`: default one physical pixel. The menu exposes 1
  through 6. Wider values enlarge the scope's cross lines.
- `cl_ironsight_usecrosshaircolor`: default false.
- `cl_ironsight_dot_scale`: 0.1 through 2, default 1.
- `cl_sniper_show_inaccuracy`: default true.
- `cl_sniper_delay_unscope`: default false. The host game controls scope transitions.
- `cl_sniper_auto_rezoom`: default true. The host game controls scope transitions.
- `cl_grenadecrosshair_flash`, `_explosive`, `_fire`, `_smoke`, `_decoy`: default true.
- `cl_grenadecrosshairdelay_flash`, `_explosive`, `_fire`, `_smoke`, `_decoy`:
  0 through 5 seconds, default 2 seconds.
- `cl_grenadecrosshair_keepusercrosshair`: default true.
- `cl_show_observer_crosshair`: off 0, friends and party 1, everyone 2; default 2.
- `cl_observed_bot_crosshair`: own 0, own when takeover is available 1,
  default crosshair 2; default 2.

The host supplies the observed player's settings, relationship, bot state, and
takeover availability. The plugin selects the displayed configuration from those
inputs. Grenade aiming depends on the host's supplied aim position and hold time.
The plugin cannot reconstruct game ballistics from the grenade kind alone.
The current guide is centered on the viewport; it does not accept a ballistic
trajectory or projected landing point.

## Text configuration contract

Use one `name value` assignment per line or separate assignments with semicolons.
Accept optional double quotes around one value. Accept `true`, `false`, `1`, or
`0` for booleans. Use decimal integer spellings for enum and color-channel values. Leading zeros
and a leading plus sign are accepted and normalized.
Use finite decimal numbers, including exponent notation, for continuous values. Reject missing values, extra
tokens, unknown names, non-finite numbers, and out-of-range values.

Parsing starts from current defaults. Repeated settings use the last assignment.
An invalid assignment rejects the entire parsed configuration. An invalid direct
update leaves the previous configuration unchanged. Export emits each supported
setting once, in a fixed documented order, with newline-separated assignments,
Rust Display number spellings, and boolean values spelled `true` or `false`.
Order follows the declarations in `src/settings.rs`, beginning with `crosshair`
and ending with `cl_observed_bot_crosshair`.
Parsing and export preserve values, rather than their original whitespace or
numeric spelling. Comments, aliases, executable console commands, and share codes
are outside this profile and must fail explicitly.

Hidden legacy `cl_crosshairalpha`, `cl_crosshairsize`, and `cl_crosshairthickness`
are migration remnants in the current dump. Their undocumented migration behavior
is not an active setting contract. Reject them with an explanation naming the
current replacement. Older `cl_crosshairgap`, `cl_crosshaircolor`,
`cl_crosshairusealpha`, `cl_crosshair_outlinethickness`, and
`cl_crosshairgap_useweaponvalue` are outside the selected current-client profile.

Development-only grenade tick-label and tick-scaling commands are excluded.
They are not user crosshair settings in the selected client menu.

## Conformance obligations

Test every default, lower bound, upper bound, invalid bound, missing value,
malformed value, enum variant, color channel, boolean spelling, duplicate, and
round trip independently. Verify style-specific control applicability, physical
pixel scaling, the one-pixel minimum, opacity, outlines, T shape, split geometry,
split alpha, supplied recoil, static versus dynamic input, and disabled output.

Test each grenade independently at times before, at, and after its configured
delay. Test scope transitions, friendly warning, spectator relationship, bot
takeover, and observed-configuration selection. Benchmark the same reachable
production branches and measure instrumented line coverage independently.

## Reconstruction boundaries

These choices are implemented and tested locally. They still require comparison
with actual CS2 captures before exact compatibility can be claimed.

- Cross and legacy split dimensions round to physical pixels. Radial gaps clamp
  to zero after spread is added. A negative static gap therefore does not produce
  overlapping bars. This is a known boundary of this reconstruction.
- Dynamic styles use host-supplied radial spread directly, capped at
  `(128 + spread_limit) × viewport_height / authored_height` physical pixels.
  Shot-feedback style uses host-supplied shot distance. No CS2 weapon model,
  temporal smoothing, animation curve, or recoil predictor is copied.
- Circle styles use rings. Quad styles use four angular ring segments. Their arc
  placement and length mapping are reconstructed. Full outlines expand one pixel;
  half outlines use half-pixel placement. Exact rasterization remains unverified.
- Scope lines span the viewport. Inaccuracy is a translucent ring whose radius
  equals supplied spread. Optical sights use a square dot of
  `3 × cl_ironsight_dot_scale` physical pixels. These shapes are reconstructions.
- Grenade guides use one horizontal line, one vertical line, and four ticks at
  one eighth and one quarter of viewport height above and below center. Tick width
  is 12 physical pixels. These are alignment guides, not ballistic predictions.
- Friendly aim draws an orange exclamation mark above supplied recoil. It does
  not include Valve artwork or reproduce the original warning icon.
- Scope transitions are host-owned states. Delay-unscope selects scoped geometry
  during cycling; auto-rezoom selects scoped geometry after cycling. The plugin
  cannot drive the host weapon controller.
- The sniper-width parser follows the menu's 1–6 range. This deliberately rejects
  values outside the menu even if another CS2 console build accepts them.
- The global visibility setting suppresses ordinary crosshair, scope, grenade,
  and friendly-warning geometry. Custom hit feedback has independent visibility.

`tests/support/configuration.rs` covers configuration boundaries and export.
`geometry.rs`, `policies.rs`, and `markers.rs` in that directory cover geometry,
host-state selection, and feedback. `events.rs` checks independent event delivery,
replacement, expiry, missing targets, and shared timing. `reflection.rs` verifies
that reflection cannot bypass validated settings. `rendering.rs` creates the real
material pipeline and exercises every style. Criterion executes these same
conformance cases for its independent instrumented line-coverage gate.
