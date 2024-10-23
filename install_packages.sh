#!/bin/bash

pkgs=(libgtk-4-dev build-essential libadwaita-1-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev libgstreamer-plugins-bad1.0-dev libgtk-4-media-gstreamer)
sudo apt-get -y --ignore-missing install "${pkgs[@]}"

