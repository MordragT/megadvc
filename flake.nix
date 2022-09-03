{
  description = "Rust dev with nix template repo";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, ... }:
    # Output schema from https://nixos.wiki/wiki/Flakes
    flake-utils.lib.eachDefaultSystem (system:
      let
        pname = "template";
        version = "0.1.0";
        pkgs = import nixpkgs { inherit system; };
      in
      {
        # called when `nix build` / `nix run` is invoked
        defaultPackage = (pkgs.makeRustPlatform {

          # use nightly rustc and cargo provided by fenix to run/build stuff 
          # options: minimal | complete
          inherit (fenix.packages.${system}.minimal) cargo rustc;
        }).buildRustPackage {
          pname = pname;
          version = version;
          src = ./.;

          # cargohash isn't taken: https://github.com/nix-community/fenix/issues/70#issuecomment-1114333311
          cargoLock.lockFile = ./Cargo.lock;

          buildInputs = with pkgs; [ megacmd ];
              
          # for other makeRustPlatform features see: 
          # https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md#cargo-features-cargo-features
        };

        # called when `nix develop` is invoked
        devShells.default = pkgs.mkShell {

          # use nightly cargo & rustc provided by fenix. Add for packages for the dev shell here
          buildInputs = (with fenix.packages.${system}.complete; [
              cargo
              rustc
              rust-src
              clippy
              rustfmt
            ]) ++
            (with pkgs; [
              pkg-config
              megacmd
            ]);

          # specify the rust-src path (many editors rely on this)
          RUST_SRC_PATH = "${fenix.packages.${system}.complete.rust-src}/lib/rustlib/src/rust/library";
        };
      }
    );
}
