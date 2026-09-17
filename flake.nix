{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell rec {
            nativeBuildInputs = [
              openssl
              stdenv.cc.cc.lib
            ];
            buildInputs = [
              pkg-config
            ];
            packages = [
              mdbook
              gh
              mdbook-mermaid
            ];
            shellHook = ''
              export LIBCLANG_PATH="${lib.makeLibraryPath [ llvmPackages_latest.libclang.lib ]}"
              export LD_LIBRARY_PATH="'$LD_LIBRARY_PATH:${lib.makeLibraryPath nativeBuildInputs}"
              PKG_CONFIG_PATH="${openssl.dev}/lib/pkgconfig";
            '';
          };
      }
    );
}
