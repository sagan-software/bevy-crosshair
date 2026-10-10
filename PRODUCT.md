# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Rust and Bevy 0.20, compiled to WebAssembly. GitHub Pages hosts the playground.
The reusable plugin also supports native Bevy applications and headless tests.
Use standard HTML controls around the Bevy canvas, with no JavaScript UI framework.
This implementation choice follows the user's delegated design defaults.

## Users

Bevy developers configure crosshairs and evaluate hit feedback before integrating
the plugin into a game. The playground exposes the same settings as the Rust API.

## Product purpose

Render configurable crosshairs with current Counter-Strike 2 settings and
metadata-driven hit markers. Independent Bevy observers can use hit metadata for
sound, score, and other game behavior.

## Capabilities and constraints

Settings must change the running Rust renderer. Display measured frame rate.
Provide a usable desktop and mobile layout. Keep controls and the crosshair visible
immediately. Keep technical design explanations in repository documentation.
CS2 compatibility requires documented limits where actual-client evidence is absent.
The requested performance target is at least 60 FPS on the verification device.

## Brand commitments

The repository is `sagan-software/Bevy-Crosshair`. Preserve the existing icon asset.
