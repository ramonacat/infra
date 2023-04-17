{
  description = "A basic flake with a shell";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }: {
    formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
    devShells.x86_64-linux.default = nixpkgs.legacyPackages.x86_64-linux.mkShell {
      shellHook = ''
        rustup default stable
      '';
      packages = with nixpkgs.legacyPackages.x86_64-linux; [
        terraform
        rustup
      ];
    };
    nixosConfigurations = {
      jump = nixpkgs.lib.nixosSystem {
        system = "aarch64-linux";
        modules = [
          ./machines/jump.nix
        ];
      };
    };
  };
}
