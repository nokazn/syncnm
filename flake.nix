{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nix-community/naersk/master";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          nativeBuildInputs = [
            pkg-config
          ];
          buildInputs = [
            cargo
            rustc
            rustfmt
            rustPackages.clippy
            cargo-watch
            pre-commit
            nixpkgs-fmt
            just
            openssl.dev
            libiconv
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          OPENSSL_INCLUDE_DIR = (
            lib.makeSearchPathOutput "dev" "include" [ pkgs.openssl.dev ]
          ) + "/openssl";
          OPENSSL_STATIC = "0";
        };
      });
}
