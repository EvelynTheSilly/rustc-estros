# Hermetic rustc-estros toolchain package.
#
# Builds the fork's stage-2 compiler + std completely inside the nix sandbox
# (no network) and installs a toolchain that ships `aarch64-estros` as a normal
# target with std. Network is confined to the fixed-output vendoring
# derivation below, so consumers evaluate and build purely.
#
# Deliberately NOT sharing the bootstrap config with `devshell.nix`: the
# devshell is intentionally non-hermetic (CI LLVM, incremental) while this
# build is fully offline against a nixpkgs-provided LLVM.
{
  self,
  pkgs,
  lib,
  system,
  channel,
  toolchain,
}:
let
  hostTriple = "x86_64-unknown-linux-gnu";
  estrosTriple = "aarch64-estros";
  llvmPackages = pkgs.llvmPackages_21;

  # Flake source minus the in-tree `x vendor` output. vendor/ is currently
  # staged in the git index (cleanup pending); excluding it here keeps the
  # build source sane (~700 MB instead of ~2.6 GB) and is a no-op once the
  # index is cleaned. We generate our own vendor/ via the FOD below anyway.
  srcRoot = self.outPath;
  cleanSrc = builtins.path {
    path = srcRoot;
    name = "rustc-estros-src";
    filter =
      path: type: !(type == "directory" && path == srcRoot + "/vendor");
  };

  # git-commit-info (sha / short_sha / commit_date) so `rustc --version` gets a
  # proper hash/date instead of falling back to a bare version string. The flake
  # source has no .git, so we synthesize it from the flake's own rev metadata.
  rev = self.dirtyRev or self.rev or "unknown";
  shortRev = self.shortRev or (builtins.substring 0 9 rev);

  # Cargo source-replacement configs pointing at the vendored crates.
  cargoConfig =
    (pkgs.formats.toml { }).generate "cargo-config.toml"
      {
        source = {
          crates-io.replace-with = "vendored-sources";
          "vendored-sources".directory = "vendor";
        };
      };
  cargoConfigLibrary =
    (pkgs.formats.toml { }).generate "cargo-config-library.toml"
      {
        source = {
          crates-io.replace-with = "vendored-sources";
          "vendored-sources".directory = "../vendor";
        };
      };

  # Hermetic bootstrap config for this package (kept private to this file).
  bootstrapConfig =
    (pkgs.formats.toml { }).generate "bootstrap.toml"
      {
        change-id = "ignore";
        build = {
          rustc = "${toolchain}/bin/rustc";
          cargo = "${toolchain}/bin/cargo";
          vendor = true;
          docs = false;
          profiler = false;
          "optimized-compiler-builtins" = false;
          "patch-binaries-for-nix" = true;
          extended = false;
          target = [
            hostTriple
            estrosTriple
          ];
        };
        llvm = {
          "download-ci-llvm" = false;
          "link-shared" = true;
        };
        rust = {
          incremental = false;
          "deny-warnings" = false;
          rpath = true;
        };
        target = {
          "${hostTriple}" = {
            "llvm-config" = "${llvmPackages.llvm.dev}/bin/llvm-config";
          };
        };
      };

  # Fixed-output derivation: vendor the crates.io dependencies for every
  # workspace the build actually uses (root, bootstrap, library). Fixed-output
  # derivations are the only place network is allowed; the real build below
  # never touches the network.
  vendorDeps = pkgs.stdenvNoCC.mkDerivation {
    pname = "rustc-estros-vendor-deps";
    version = channel;
    src = cleanSrc;

    nativeBuildInputs = [
      toolchain
      pkgs.cacert
    ];

    outputHashMode = "recursive";
    outputHashAlgo = "sha256";
    outputHash = "sha256-SeD7EwEnICV6zGaP7yhM83ozR5BonUtL9RNfaK3x+8g=";

    dontConfigure = true;
    dontFixup = true;

    buildPhase = ''
      runHook preBuild

      export HOME=$TMPDIR
      export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
      export CARGO_HOME=$TMPDIR/cargo-home
      mkdir -p "$CARGO_HOME" "$out"

      export RUSTC_BOOTSTRAP=1
      export RUSTC=${toolchain}/bin/rustc

      ${toolchain}/bin/cargo vendor \
        --sync Cargo.toml \
        --sync src/bootstrap/Cargo.toml \
        --sync library/Cargo.toml \
        "$out/vendor" > /dev/null

      runHook postBuild
    '';
  };
in
pkgs.stdenv.mkDerivation {
  pname = "rustc-estros";
  version = channel;
  src = cleanSrc;

  nativeBuildInputs = [
    toolchain
    pkgs.python3
    pkgs.pkg-config
    pkgs.cmake
    pkgs.ninja
    pkgs.file
    pkgs.which
    pkgs.patchelf
    pkgs.git
    pkgs.xz
    pkgs.gnutar
    pkgs.cacert
  ];

  buildInputs = [
    llvmPackages.llvm
    pkgs.zlib
    pkgs.openssl
    pkgs.libffi
    pkgs.libxml2
  ];

  # No configure step; bootstrap drives everything from bootstrap.toml.
  dontConfigure = true;
  # `./x.py install` both builds and installs into $out, so the default
  # make-based installPhase is disabled.
  installPhase = "true";

  buildPhase = ''
    runHook preBuild

    export HOME=$TMPDIR
    export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
    export RUST_BOOTSTRAP_CONFIG=${bootstrapConfig}

    # Stage the vendored crate sources so cargo runs fully offline. bootstrap
    # passes --frozen to every cargo call when build.vendor = true, and
    # bootstrap.py refuses to start unless vendor/ and .cargo/ both exist.
    mkdir -p .cargo library/.cargo
    cp -a ${vendorDeps}/vendor ./vendor
    cp -a ${cargoConfig} .cargo/config.toml
    cp -a ${cargoConfigLibrary} library/.cargo/config.toml

    # Synthesize git-commit-info (sha / short_sha / commit_date) since the flake
    # source carries no .git directory.
    printf '%s\n%s\n%s\n' \
      "${rev}" \
      "${shortRev}" \
      "$(date -u -d "@''${SOURCE_DATE_EPOCH:-$(git log -1 --date=short --pretty=format:%ct 2>/dev/null || echo 0)}" +%Y-%m-%d 2>/dev/null || date -u -d "@0" +%Y-%m-%d)" \
      > git-commit-info

    # bootstrap's x.py has a `#!/usr/bin/env` shebang the sandbox lacks; drive
    # it through python3 directly.
    # `install.sysconfdir` must be a *relative* path: bootstrap does
    # `prefix.join(sysconfdir)`, and an absolute default ("/etc") would replace
    # the prefix and make it check /etc for writability.
    ${pkgs.python3}/bin/python3 ./x.py install --stage 2 \
      --set "install.prefix=$out" \
      --set "install.sysconfdir=etc"

    runHook postBuild
  '';

  # Point the output at the nixpkgs LLVM so the closure doesn't drag in the
  # full build tree. `patch-binaries-for-nix = true` already gives the binaries
  # a correct rpath to libLLVM.so at build time.
  postFixup = ''
    # Drop any stray references to the temporary build directory.
    for f in "$out"/bin/* "$out"/lib/*.so "$out"/libexec/*; do
      [ -e "$f" ] && remove-references-to -t "$NIX_BUILD_TOP" "$f" 2>/dev/null || true
    done
  '';
}
