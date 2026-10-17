{
  description = "estros: dev shell and packages for a rustc fork";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/d6c71932130818840fc8fe9509cf50be8c64634f";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      inherit (pkgs) lib;

      stage0 = builtins.readFile ./src/stage0;
      stage0Lines = lib.splitString "\n" stage0;
      stage0Value =
        name:
        lib.removePrefix "${name}=" (lib.head (lib.filter (line: lib.hasPrefix "${name}=" line) stage0Lines));
      channel = "${stage0Value "compiler_version"}-${stage0Value "compiler_date"}";

      toolchain = pkgs.rust-bin.fromRustupToolchain {
        inherit channel;
        profile = "minimal";
        components = [
          "rustc"
          "cargo"
          "rustfmt"
        ];
      };

      args = {
        inherit
          self
          pkgs
          lib
          system
          channel
          toolchain
          ;
      };
    in
    {
      devShells.${system}.default = import ./devshell.nix args;
      packages.${system}.default = import ./rustc_package.nix args;
    };
}
