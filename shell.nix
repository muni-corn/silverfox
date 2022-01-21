{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  buildInputs = builtins.attrValues {
    inherit (pkgs) cargo cargo-watch;
  } ++ [ pkgs.rust-bin.nightly.latest.default ];
}
