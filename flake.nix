{
  description = "A basic flake with a shell";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay, crane }:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit overlays; system = "x86_64-linux"; };
      rustVersion = pkgs.rust-bin.stable.latest.default;
      craneLib = (crane.mkLib pkgs).overrideToolchain rustVersion;
      sourceFilter = path: type: (builtins.match ".*/.sqlx/.*" path != null) || (builtins.match ".*/migrations/.*" path != null) || craneLib.filterCargoSources path type;
      packageArguments = {
        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./apps/backend;
          filter = sourceFilter;
        };
      };
      cargoArtifacts = craneLib.buildDepsOnly packageArguments;
      backendPackage = craneLib.buildPackage (packageArguments // {
        inherit cargoArtifacts;
      });
    in
    {
      formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
      devShells.x86_64-linux.default = nixpkgs.legacyPackages.x86_64-linux.mkShell {
        shellHook = ''
          cargo install sqlx-cli
        '';
        packages = with nixpkgs.legacyPackages.x86_64-linux; [
          terraform
          fluxcd
          pkgconfig
          openssl
          (pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          })
        ];
      };
      checks.x86_64-linux = {
        inherit backendPackage;

        backendPackageClippy = craneLib.cargoClippy (packageArguments // { inherit cargoArtifacts; });
        backendPackageFmt = craneLib.cargoFmt (packageArguments // { inherit cargoArtifacts; });
      };
      packages.x86_64-linux =
        {
          backend = pkgs.dockerTools.buildImage
            {
              name = "backend";
              tag = "default";
              config = {
                contents = [ pkgs.cacert ];
                Cmd = [ "${backendPackage}/bin/backend" ];
                Labels = {
                  "org.opencontainers.image.source" = "https://github.com/ramonacat/infra";
                };
              };
            };
          backend-migrations = pkgs.dockerTools.buildImage
            {
              name = "backend-migrations";
              tag = "default";
              config = {
                contents = [ pkgs.cacert ];
                Cmd = [ "${backendPackage}/bin/migrate" ];
                Labels = {
                  "org.opencontainers.image.source" = "https://github.com/ramonacat/infra";
                };
              };
            };
        };
    };
}
