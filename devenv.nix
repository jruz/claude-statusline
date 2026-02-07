{ pkgs, ... }:

let nu-lint = import ./nu-lint.nix { inherit pkgs; };
in {
  packages = with pkgs;
    [
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
      statix
      taplo
      woodpecker-cli
      yamllint
    ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ libiconv ]
    ++ pkgs.lib.optionals pkgs.stdenv.isLinux [ glibc glibcLocales ];

  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
  };

  enterShell = ''
    echo ""
    echo "  Rust Development Shell"
    echo "  ───────────────────────────"
    echo "  Rust:        $(rustc --version | cut -d' ' -f2)"
    echo "  Nextest:     ${pkgs.cargo-nextest.version}"
    echo "  Taplo:       ${pkgs.taplo.version}"
    echo "  Woodpecker:  ${pkgs.woodpecker-cli.version}"
    echo ""
  '' + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
    export LOCALE_ARCHIVE="${pkgs.glibcLocales}/lib/locale/locale-archive"
  '';
}
