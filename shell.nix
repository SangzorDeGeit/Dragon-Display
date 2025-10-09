{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = [
    pkgs.pkg-config
    pkgs.openssl
    pkgs.gtk4
    pkgs.libadwaita
    pkgs.cairo
    pkgs.gdk-pixbuf
    pkgs.gst_all_1.gstreamer
    pkgs.gst_all_1.gst-plugins-base
    pkgs.gst_all_1.gst-plugins-good
    pkgs.gst_all_1.gst-plugins-bad
    pkgs.gst_all_1.gst-plugins-ugly
    pkgs.gst_all_1.gst-libav
    pkgs.gst_all_1.gst-vaapi
  ];
}
