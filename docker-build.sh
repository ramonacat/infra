#!/usr/bin/env bash
IMAGE=$(nix build -L -v ".#$1" && docker load < result | sed 's/Loaded image: //')

docker tag "$IMAGE" "$2"