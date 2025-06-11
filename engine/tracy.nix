{ pkgs ? import <nixpkgs> {} }:

let
  nixgl = import (builtins.fetchTarball "https://github.com/nix-community/nixGL/archive/main.tar.gz") {
    pkgs = pkgs;
  };

  # Pin nixpkgs where tracy is version 1.10.0
  pinnedPkgs = import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/6494c1d470c6ccb9e565398d2b1c44f27b5b10d9.tar.gz";
  }) {};

in

pkgs.mkShell {
  buildInputs = [
    nixgl.auto.nixGLDefault
    pinnedPkgs.tracy
    # pkgs.tracy
  ];
  
  shellHook = ''
    nixGL tracy -a 127.0.0.1
    exit
  '';
}
