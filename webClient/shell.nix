# shell.nix
# WIP
{ pkgs ? import <nixpkgs> {} }:

let
  start_dev = pkgs.writeScriptBin "start_dev" ''
    #!/usr/bin/env bash
    
    clear

    # Kill background processes when the script exits
    trap 'kill $(jobs -p)' EXIT
    
    echo "Starting ASP.NET Core backend..."
    (cd ChessFlowSite.Server && export ASPNETCORE_ENVIRONMENT=Development && dotnet run --launch-profile https) &
    
    wait
  '';
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    dotnet-sdk_8
    nodejs_20
    nodePackages.npm
    sqlite
    
    start_dev
  ];

  shellHook = ''
    export ASPNETCORE_URLS="https://localhost:7073;http://localhost:5073"

    (cd chessflowsite.client && npm install) &
    
    start_dev

    exit
  '';
}
