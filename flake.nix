{
  nixConfig = {
    extra-substituters = [ "https://oxalica.cachix.org" ];
    extra-trusted-public-keys =
      [ "oxalica.cachix.org-1:xIQx5E4KAt0BXOY0TFfCMx+JsMhMVID0bW70GBCElQ0=" ];
  };

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
        rustVersion = "1.92.0";
        rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
          extensions = [ "rust-analyzer" "rust-src" ];
        };
        nu-lint-version = "0.0.142";
        nu-lint = if pkgs.stdenv.isLinux then
          let
            arch = if pkgs.stdenv.hostPlatform.isAarch64 then
              "aarch64"
            else
              "x86_64";
            sha256 = if pkgs.stdenv.hostPlatform.isAarch64 then
              "6d6239e28ac67067b15c92ede1a103da20cd7bae9cd4da716c9109a35a8d9ad4"
            else
              "e4ba5bccbfb511494033b2d7dceba09a2ee72807d3426112426521de46a4233b";
          in pkgs.stdenv.mkDerivation {
            pname = "nu-lint";
            version = nu-lint-version;
            src = pkgs.fetchurl {
              url =
                "https://codeberg.org/wvhulle/nu-lint/releases/download/v${nu-lint-version}/nu-lint-${arch}-unknown-linux-musl.tar.gz";
              inherit sha256;
            };
            nativeBuildInputs = [ pkgs.gnutar ];
            phases = [ "unpackPhase" "installPhase" ];
            unpackPhase = ''
              tar -xzf $src
            '';
            installPhase = ''
              mkdir -p $out/bin
              cp nu-lint $out/bin/
              chmod +x $out/bin/nu-lint
            '';
          }
        else
          pkgs.rustPlatform.buildRustPackage {
            pname = "nu-lint";
            version = nu-lint-version;
            src = pkgs.fetchgit {
              url = "https://codeberg.org/wvhulle/nu-lint.git";
              rev = "v${nu-lint-version}";
              sha256 = "sha256-Hdm1JtChg5dSnPKO8q2EVVGUL4978ZK97trvi9J1rIc=";
            };
            cargoHash = "sha256-yqF+jDaQ7DwLRUZvvwsTRDmeB70Qsw0ywO4RzcGrXRo=";
          };
        commonPackages = with pkgs; [
          cargo-edit
          cargo-nextest
          cargo-outdated
          cargo-watch
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
          sqlx-cli
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
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = platformPackages;

          shellHook = ''
            echo ""
            echo "  Rust Development Shell"
            echo "  ───────────────────────────"
            echo "  Rust:        ${rustVersion}"
            echo "  Nextest:     ${pkgs.cargo-nextest.version}"
            echo "  Taplo:       ${pkgs.taplo.version}"
            echo "  Woodpecker:  ${pkgs.woodpecker-cli.version}"
            echo ""
          '' + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
            export LOCALE_ARCHIVE="${pkgs.glibcLocales}/lib/locale/locale-archive"
          '';
        };
      });
}
