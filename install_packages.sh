#!/bin/bash

pkgs=(libgtk-4-dev build-essential)
sudo apt-get -y --ignore-missing install "${pkgs[@]}"

