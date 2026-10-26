{
  description = "Weave — portable, renderer-independent expressive motion engine (Rust)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      # Weave is pure Rust and portable: the *core* builds everywhere.
      # Adapters are optional, so all supported systems share one shell.
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forEachSupportedSystem = f:
        nixpkgs.lib.genAttrs supportedSystems (system: f {
          inherit system;
          pkgs = nixpkgs.legacyPackages.${system};
        });

      # ---------------------------------------------------------------------------
      # Native dependencies.
      #
      # The portable core (weave-core / weave-geometry / weave-scene) and the
      # terminal demos need NOTHING from this list. These packages are only
      # required by the optional adapters:
      #   * weave-wgpu  -> vulkan loader, libGL, libxkbcommon, X11/wayland clients
      #   * weave-web   -> wasm-bindgen / wasm-pack (pure Rust, no native deps)
      # ---------------------------------------------------------------------------
      nativeDeps = pkgs: with pkgs; [
        pkg-config

        # Rendering (optional weave-wgpu adapter)
        vulkan-loader
        libGL
        libxkbcommon
        wayland
        libX11
      ];

      # Only the files needed to build the workspace. A fileset keeps the
      # (multi-GB) target/ directory and .git out of the Nix store.
      projectFiles = pkgs: pkgs.lib.fileset.unions [
        ./Cargo.toml
        ./Cargo.lock
        ./README.md
        ./LICENSE-MIT
        ./LICENSE-APACHE
        ./crates
        ./examples
      ];

      rustTools = pkgs: with pkgs; [
        cargo
        rustc
        rustfmt
        clippy
        rust-analyzer
        rustup
      ];

      mkDevShell = pkgs: pkgs.mkShell {
        name = "weave-dev";

        nativeBuildInputs = nativeDeps pkgs ++ (rustTools pkgs) ++ (with pkgs; [
          nixfmt
          wasm-pack
        ]);

        # Driver libraries (GL, Vulkan, input) are dlopen'ed at runtime.
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (nativeDeps pkgs);

        RUST_BACKTRACE = "1";

        shellHook = ''
          echo "Weave dev shell"
          echo "  cargo build --workspace            # portable core + terminal demos"
          echo "  cargo test --workspace             # unit tests"
          echo "  cargo clippy --workspace --all-targets -- -D warnings"
          echo "  cargo run -p spring-playground     # interactive spring playground"
          echo "  cargo run -p button-to-panel       # pill -> panel morph demo"
          echo ""
          echo "  rustup target add wasm32-unknown-unknown   # only needed for weave-web"
          echo "  nix flake check                    # build + test in the Nix sandbox"
          echo ""
        '';
      };
    in
    {
      devShells = forEachSupportedSystem ({ pkgs, ... }: {
        default = mkDevShell pkgs;
        # Deterministic README GIF generation using samples from the engine.
        docs = pkgs.mkShell {
          packages = rustTools pkgs ++ [ (pkgs.python3.withPackages (ps: [ ps.pillow ])) ];
          WEAVE_DOCS_FONT = "${pkgs.dejavu_fonts}/share/fonts/truetype/DejaVuSans.ttf";
        };
      });

      # ---------------------------------------------------------------------------
      # `nix build .#` -> the portable demos.
      # ---------------------------------------------------------------------------
      packages = forEachSupportedSystem ({ pkgs, ... }: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "weave";
          version = "0.1.0";

          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = projectFiles pkgs;
          };

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = nativeDeps pkgs;

          doCheck = true;

          # The default `cargoInstallHook` puts every executable of the
          # release profile into `$out/bin`, which is exactly the two demos
          # (`spring-playground` and `button-to-panel`).

          # The demos are pure computation; nothing is dlopen'ed at build time.
          dontFixup = true;
        };
      });

      # ---------------------------------------------------------------------------
      # `nix flake check` -> build + run the whole test suite in the sandbox.
      # ---------------------------------------------------------------------------
      checks = forEachSupportedSystem ({ pkgs, ... }: {
        tests = pkgs.rustPlatform.buildRustPackage {
          pname = "weave-tests";
          version = "0.1.0";

          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = projectFiles pkgs;
          };

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = nativeDeps pkgs;

          # `doCheck = true` runs `cargo test --release --workspace` inside the
          # sandbox: the whole test suite, including doc-tests.
          doCheck = true;

          dontFixup = true;
        };
      });

      apps = forEachSupportedSystem ({ pkgs, ... }: {
        spring-playground = {
          type = "app";
          program = "${self.packages.${pkgs.system}.default}/bin/spring-playground";
          meta.description = "Interactive terminal playground for the Weave spring solver";
        };
        button-to-panel = {
          type = "app";
          program = "${self.packages.${pkgs.system}.default}/bin/button-to-panel";
          meta.description = "Pill-to-panel morph demo built on the Weave motion engine";
        };
        default = {
          type = "app";
          program = "${self.packages.${pkgs.system}.default}/bin/button-to-panel";
          meta.description = "Pill-to-panel morph demo built on the Weave motion engine";
        };
      });

      formatter = forEachSupportedSystem ({ pkgs, ... }: pkgs.nixfmt);
    };
}
