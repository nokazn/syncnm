{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nix-community/naersk/master";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat.url = "github:edolstra/flake-compat";
  };

  outputs =
    { nixpkgs
    , flake-utils
    , naersk
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };
      naersk-lib = pkgs.callPackage naersk { };
    in
    {
      packages = {
        default = naersk-lib.buildPackage ./.;
      };

      devShells = with pkgs; {
        default = mkShell {
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
      };
    });
}
