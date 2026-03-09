{
  description = "DevShell";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain =
          builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
        rustVersion = rustToolchain.toolchain.channel;
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        nu-lint = import ./nu-lint.nix { inherit pkgs; };
        commonPackages = with pkgs; [
          babashka
          cargo-edit
          cargo-nextest
          cargo-outdated
          cargo-watch
          clj-kondo
          zprint
          deadnix
          fd
          just
          markdownlint-cli2
          nixfmt-classic
          nu-lint
          nushell
          pkg-config
          ripgrep
          rust
          statix
          taplo
          woodpecker-cli
          yamllint
        ];
        darwinPackages = with pkgs; [ libiconv ];
        linuxPackages = with pkgs; [ glibc glibcLocales ];
        platformPackages = if pkgs.stdenv.isDarwin then
          commonPackages ++ darwinPackages
        else
          commonPackages ++ linuxPackages;
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        claude-statusline = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          inherit (cargoToml.package) version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [ pkg-config perl ];
          buildInputs = with pkgs;
            [ openssl ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              libiconv
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
            ];
        };
      in {
        packages.default = claude-statusline;

        devShells.default = pkgs.mkShell {
          buildInputs = platformPackages;

          shellHook = ''
            echo ""
            echo "  Rust Development Shell"
            echo "  ───────────────────────────"
            echo "  Rust:        ${rustVersion}"
            echo "  Nextest:     ${pkgs.cargo-nextest.version}"
            echo "  Taplo:       ${pkgs.taplo.version}"
            echo "  Babashka:    ${pkgs.babashka.version}"
            echo "  Woodpecker:  ${pkgs.woodpecker-cli.version}"
            echo ""
          '' + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
            export LOCALE_ARCHIVE="${pkgs.glibcLocales}/lib/locale/locale-archive"
          '';
        };
      });
}
