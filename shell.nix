{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = [
    pkgs.pkg-config
    pkgs.openssl
    pkgs.gtk4
    pkgs.libadwaita
    pkgs.cairo
    pkgs.gdk-pixbuf
  ];
}
