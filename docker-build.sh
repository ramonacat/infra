#!/usr/bin/env bash
IMAGE=$(nix build ".#$1" && docker load < result | sed 's/Loaded image: //')

docker tag "$IMAGE" "$2"