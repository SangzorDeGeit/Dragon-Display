#!/bin/bash

pkgs=(libgtk-4-dev build-essential libadwaita-1-dev libssl-dev pkg-config)
sudo apt-get -y --ignore-missing install "${pkgs[@]}"

