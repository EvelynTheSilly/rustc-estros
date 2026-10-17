{ pkgs, toolchain, ... }:
let
  inherit (pkgs) lib;

  bootstrapConfig = (pkgs.formats.toml { }).generate "bootstrap.toml" {
    change-id = "ignore";
    build = {
      rustc = "${toolchain}/bin/rustc";
      cargo = "${toolchain}/bin/cargo";
      "patch-binaries-for-nix" = true;
      docs = false;
      profiler = false;
      "optimized-compiler-builtins" = false;
    };
    llvm = {
      "download-ci-llvm" = true;
    };
    rust = {
      incremental = true;
      "deny-warnings" = false;
    };
  };
in
pkgs.mkShell {
  name = "estros";

  packages = [
    toolchain
    pkgs.git
    pkgs.nix
    pkgs.python3
    pkgs.gnutar
    pkgs.xz
    pkgs.file
    pkgs.cacert
    pkgs.curl
    pkgs.pkg-config
    pkgs.cmake
    pkgs.ninja
    pkgs.patchelf
    pkgs.libiconv
    pkgs.gcc
    pkgs.glibc.out
    pkgs.glibc.static
    pkgs.openssl
    pkgs.zlib
    pkgs.zstd
  ];

  env = {
    NIX_PATH = "nixpkgs=${pkgs.path}";
    RUST_BOOTSTRAP_CONFIG = bootstrapConfig;
    RUSTC_ICE = "0";
    RUST_BACKTRACE = "1";
    SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
    LD_LIBRARY_PATH = lib.makeLibraryPath [
      pkgs.stdenv.cc.cc.lib
      pkgs.curl
      pkgs.zlib
      pkgs.xz
    ];
    PKG_CONFIG_PATH = lib.makeSearchPathOutput "dev" "lib/pkgconfig" [
      pkgs.openssl
      pkgs.zlib
      pkgs.zstd
    ];
  };
}
