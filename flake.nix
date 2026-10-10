{
  description = "bevy-crosshair development and verification";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    dylints.url = "github:sagan-software/dylints/03fc5f93c2bcdd02a8f7f2fbe2710a315d5e06a9";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      dylints,
      ...
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      eachSystem = nixpkgs.lib.genAttrs systems;
      environment =
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          libraries = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
            pkgs.vulkan-loader
            pkgs.libxkbcommon
            pkgs.libx11
            pkgs.libxcursor
            pkgs.libxi
            pkgs.libxrandr
            pkgs.libGL
          ];
          tools = [
            rust
            pkgs.pkg-config
            pkgs.cargo-llvm-cov
            pkgs.wasm-bindgen-cli
            pkgs.python3
            pkgs.nixfmt
            pkgs.nodejs
          ];
          profilingTools = [
            pkgs.cargo-flamegraph
            pkgs.samply
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.perf ];
          lint = dylints.packages.${system}.default;
          lintShell = dylints.devShells.${system}.default;
          command =
            name: extra: text:
            pkgs.writeShellApplication {
              inherit name;
              runtimeInputs = tools ++ extra;
              text = ''
                export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath libraries}:''${LD_LIBRARY_PATH:-}"
              ''
              + text;
            };
        in
        {
          inherit
            pkgs
            tools
            profilingTools
            libraries
            lint
            lintShell
            command
            ;
        };
    in
    {
      devShells = eachSystem (
        system:
        let
          e = environment system;
        in
        {
          default = e.pkgs.mkShell {
            packages = e.tools ++ e.profilingTools;
            buildInputs = e.libraries;
            LD_LIBRARY_PATH = e.pkgs.lib.makeLibraryPath e.libraries;
          };
        }
      );
      apps = eachSystem (
        system:
        let
          e = environment system;
          app = description: drv: {
            type = "app";
            program = "${drv}/bin/${drv.name}";
            meta = { inherit description; };
          };
        in
        {
          dylint = app "Check Rust with the pinned Dylints" (
            e.command "crosshair-dylint" e.lintShell.nativeBuildInputs (
              ''
                unset CARGO_BUILD_BUILD_DIR RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER
                export RUSTUP_HOME="''${XDG_CACHE_HOME:-$HOME/.cache}/bevy-crosshair/rustup"
              ''
              + e.lintShell.shellHook
              + ''
                export DYLINT_RUSTFLAGS="-D warnings"
                libraries=("${e.lint}/lib/"*)
                exec cargo dylint --no-metadata --no-build --lib-path "''${libraries[0]}" --workspace -- --all-targets --all-features "$@"
              ''
            )
          );
          lint-probe = app "Verify that Dylints reject a known violation" (
            e.command "crosshair-lint-probe" e.lintShell.nativeBuildInputs (
              ''
                export RUSTUP_HOME="''${XDG_CACHE_HOME:-$HOME/.cache}/bevy-crosshair/rustup"
              ''
              + e.lintShell.shellHook
              + ''
                libraries=("${e.lint}/lib/"*)
                exec python3 scripts/lint_probe.py "''${libraries[0]}"
              ''
            )
          );
        }
      );
      formatter = eachSystem (system: (environment system).pkgs.nixfmt);
      checks = eachSystem (
        system:
        let
          e = environment system;
        in
        {
          metadata = e.pkgs.runCommand "crosshair-lint-metadata" { } ''
            test -d ${dylints}/crates/sagan-lints
            test "${dylints.rev}" = "03fc5f93c2bcdd02a8f7f2fbe2710a315d5e06a9"
            touch "$out"
          '';
        }
      );
    };
}
