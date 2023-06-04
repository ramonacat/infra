#!/usr/bin/env bash
IMAGE=$(nix build ".#backend" && docker load < result | sed 's/Loaded image: //')

docker tag "$IMAGE" "$1"