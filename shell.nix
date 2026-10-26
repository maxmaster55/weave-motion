# Standalone shell for people who prefer `nix-shell` over flakes.
# The flake (flake.nix) is the primary entry point: `nix develop`.
#
# Usage:
#   nix-shell
#   nix-shell --run 'cargo test --workspace'
{ pkgs ? import <nixpkgs> { } }:

let
  nativeDeps = with pkgs; [
    pkg-config

    # Rendering (optional weave-wgpu adapter)
    vulkan-loader
    libGL
    libxkbcommon
    wayland
    libX11
  ];
in
pkgs.mkShell {
  name = "weave-dev";

  nativeBuildInputs = nativeDeps ++ (with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
    rustup
    nixfmt
    wasm-pack
  ]);

  shellHook = ''
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath nativeDeps}:$LD_LIBRARY_PATH"
    export RUST_BACKTRACE=1
    echo "Weave dev shell (nix-shell)"
    echo "  cargo build --workspace"
    echo "  cargo test --workspace"
    echo "  cargo run -p spring-playground"
    echo "  cargo run -p button-to-panel"
    echo ""
  '';
}
