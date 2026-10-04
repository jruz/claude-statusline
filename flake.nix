{
  description = "Claude Code statusline";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-26.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustToolchain =
          builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
        rustVersion = rustToolchain.toolchain.channel;
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        nu-lint = import ./nu-lint.nix { inherit pkgs; };
        revision = self.rev or "dirty";
        sandboxPath = pkgs.lib.makeBinPath [
          rust
          pkgs.bash
          pkgs.bun
          pkgs.cargo-edit
          pkgs.cargo-nextest
          pkgs.cargo-outdated
          pkgs.cargo-watch
          pkgs.charm-freeze
          pkgs.coreutils
          pkgs.findutils
          pkgs.git
          pkgs.gnugrep
          pkgs.gnused
          pkgs.just
          pkgs.nushell
          pkgs.stdenv.cc
          pkgs.woodpecker-cli
        ];
        sandbox = pkgs.writeShellScriptBin "sandbox" ''
          SELF_DIR="$(dirname "$(readlink -f "$0")")"
          SANDBOX_CMD="''${@:-$SHELL}"
          ARGS=(
            --unshare-user
            --unshare-ipc
            --unshare-uts
            --unshare-cgroup
            --share-net
            --die-with-parent
            --ro-bind /nix /nix
            --ro-bind /etc/resolv.conf /etc/resolv.conf
            --ro-bind /etc/ssl /etc/ssl
            --ro-bind-try /etc/static /etc/static
            --ro-bind-try /etc/nix /etc/nix
            --ro-bind-try /run/current-system/sw /run/current-system/sw
            --symlink ${pkgs.coreutils}/bin/env /usr/bin/env
            --symlink ${pkgs.bash}/bin/bash /bin/sh
            --tmpfs "$HOME"
            --bind "$(pwd)" "$(pwd)"
            --bind /tmp /tmp
            --ro-bind "$(git rev-parse --git-common-dir)" "$(git rev-parse --git-common-dir)"
            --ro-bind-try "$HOME/.gitconfig" "$HOME/.gitconfig"
            --ro-bind-try "$HOME/.ssh/known_hosts" "$HOME/.ssh/known_hosts"
            --setenv PATH "${sandboxPath}:$SELF_DIR"
            --setenv HOME "$HOME"
            --setenv REVISION "''${REVISION:-dev}"
            --dev /dev
            --proc /proc
          )
          ARGS+=(
            --bind-try "$HOME/.cargo" "$HOME/.cargo"
            --bind-try "$HOME/.bun" "$HOME/.bun"
          )
          exec ${pkgs.bubblewrap}/bin/bwrap "''${ARGS[@]}" -- "$@"
        '';
        defaultPackages = with pkgs;
          [
            babashka
            clj-kondo
            deadnix
            fd
            just
            markdownlint-cli2
            nixfmt-classic
            nu-lint
            nushell
            ripgrep
            sandbox
            statix
            taplo
            woodpecker-cli
            yamllint
            zprint
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [ glibc glibcLocales ];
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
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

        devShells.default = pkgs.mkShell {
          buildInputs = defaultPackages;

          shellHook = ''
            echo ""
            echo "  ${cargoToml.package.name} Development Shell"
            echo "  ───────────────────────────"
            echo "  rustc ${rustVersion} (sandbox)"
            echo "  bun ${pkgs.bun.version} (sandbox)"
            echo ""
          '' + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
            export LOCALE_ARCHIVE="${pkgs.glibcLocales}/lib/locale/locale-archive"
          '';
        };
      });
}
