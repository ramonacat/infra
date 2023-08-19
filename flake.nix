{
  description = "A basic flake with a shell";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit overlays; system = "x86_64-linux"; };
      rustVersion = pkgs.rust-bin.stable.latest.default;
    in
    {
      formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
      devShells.x86_64-linux.default = nixpkgs.legacyPackages.x86_64-linux.mkShell {
        packages = with nixpkgs.legacyPackages.x86_64-linux; [
          terraform
          fluxcd
          (pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          })
        ];
      };

      packages.x86_64-linux =
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit overlays; system = "x86_64-linux"; };
          rustVersion = pkgs.rust-bin.stable.latest.default;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustVersion;
            rustc = rustVersion;
            cargo-auditable = pkgs.cargo-auditable;
          };
          backendPackage = rustPlatform.buildRustPackage {
            pname = "backend";
            version = "0.1.0";
            src = ./apps/backend;
            cargoLock.lockFile = ./apps/backend/Cargo.lock;
          };
        in
        {
          backend = pkgs.dockerTools.buildImage
            {
              name = "backend";
              config = {
                Cmd = [ "${backendPackage}/bin/backend" ];
                Labels = {
                  "org.opencontainers.image.source" = "https://github.com/Agares/infra";
                };
              };
            };
          backend-migrations = pkgs.dockerTools.buildImage
            {
              name = "backend";
              config = {
                Cmd = [ "${backendPackage}/bin/migrate" ];
                Labels = {
                  "org.opencontainers.image.source" = "https://github.com/Agares/infra";
                };
              };
            };
        };
    };
}
