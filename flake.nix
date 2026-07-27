{
  description = "Joséphine - your computer's quiet guardian spirit";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      # Josephine is Linux-only (systemd, /sys, /proc, libnotify), so we only
      # expose Linux systems.
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f {
            inherit system;
            pkgs = nixpkgs.legacyPackages.${system};
          }
        );

      modules = import ./packaging/nix/module.nix;
    in
    {
      packages = forAllSystems (
        { pkgs, ... }:
        let
          josephine = pkgs.callPackage ./packaging/nix/package.nix { };
        in
        {
          inherit josephine;
          default = josephine;
        }
      );

      apps = forAllSystems (
        { system, ... }:
        let
          josephine = {
            type = "app";
            program = nixpkgs.lib.getExe self.packages.${system}.josephine;
          };
        in
        {
          inherit josephine;
          default = josephine;
        }
      );

      overlays.default = _final: prev: {
        josephine = prev.callPackage ./packaging/nix/package.nix { };
      };

      devShells = forAllSystems (
        { system, pkgs, ... }:
        {
          default = pkgs.mkShell {
            # Pull in the crate's build inputs (rustc, cargo, the C toolchain for
            # the bundled SQLite), then add the tools the quality gate needs.
            inputsFrom = [ self.packages.${system}.josephine ];
            packages = [
              pkgs.clippy
              pkgs.rustfmt
              pkgs.rust-analyzer
            ];
          };
        }
      );

      nixosModules.default = modules.nixos;
      homeManagerModules.default = modules.homeManager;

      # VM test of the NixOS module (needs KVM, so `nix flake check` runs it on
      # a Linux host with /dev/kvm).
      checks = forAllSystems (
        { pkgs, ... }:
        {
          nixos-module = import ./packaging/nix/nixos-test.nix {
            inherit pkgs;
            module = self.nixosModules.default;
          };
        }
      );

      formatter = forAllSystems ({ pkgs, ... }: pkgs.nixfmt);
    };
}
