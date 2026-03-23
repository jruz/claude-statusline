{
  description = "Claude Code statusline";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-25.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      systems =
        [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system:
          f {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            };
          });
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      revision = self.rev or "dirty";
    in {
      packages = forAllSystems ({ pkgs }: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          inherit (cargoToml.package) version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          REVISION = revision;
          CARGO_HOME = "$TMPDIR";
          doCheck = false;
          buildInputs =
            pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
        };
      });

      devShells = forAllSystems ({ pkgs }:
        let nu-lint = import ./nu-lint.nix { inherit pkgs; };
        in {
          default = pkgs.mkShell {
            packages = with pkgs;
              [
                (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
                babashka
                bun
                cargo-edit
                cargo-nextest
                cargo-outdated
                cargo-watch
                clj-kondo
                deadnix
                fd
                just
                markdownlint-cli2
                nixfmt-classic
                nu-lint
                nushell
                ripgrep
                statix
                taplo
                woodpecker-cli
                yamllint
                zprint
              ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ libiconv ]
              ++ pkgs.lib.optionals pkgs.stdenv.isLinux [ glibc glibcLocales ];

            shellHook = ''
              echo ""
              echo "  ${cargoToml.package.name} Development Shell"
              echo "  ───────────────────────────"
              echo "  rustc $(rustc --version | cut -d' ' -f2)"
              echo ""
            '' + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
              export LOCALE_ARCHIVE="${pkgs.glibcLocales}/lib/locale/locale-archive"
            '';
          };
        });
    };
}
