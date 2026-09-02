# NixOS and Home Manager modules for Josephine.
#
# Both run the background watcher as a systemd *user* service: the daemon keeps
# its config under ~/.config/josephine and its history under
# ~/.local/share/josephine, so it belongs to the user session rather than the
# system. The unit mirrors packaging/systemd/josephine.service, with ExecStart
# pointed at the package in the Nix store instead of /usr/bin.
let
  # Shared identity, kept in one place so the two module flavours cannot drift.
  serviceDescription = "Joséphine - your computer's guardian angel";
  documentationUrl = "https://github.com/systm-d/josephine";

  packageOption =
    { lib, pkgs }:
    lib.mkOption {
      type = lib.types.package;
      default = pkgs.callPackage ./package.nix { };
      defaultText = lib.literalExpression "pkgs.callPackage ./package.nix { }";
      description = "The josephine package to install and run.";
    };
in
{
  # NixOS: install system-wide and register the watcher for every user session.
  nixos =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      cfg = config.services.josephine;
    in
    {
      options.services.josephine = {
        enable = lib.mkEnableOption "the Josephine system guardian (user daemon)";
        package = packageOption { inherit lib pkgs; };
      };

      config = lib.mkIf cfg.enable {
        environment.systemPackages = [ cfg.package ];

        systemd.user.services.josephine = {
          description = serviceDescription;
          documentation = [ documentationUrl ];
          after = [ "graphical-session.target" ];
          wantedBy = [ "default.target" ];
          serviceConfig = {
            Type = "simple";
            ExecStart = "${lib.getExe cfg.package} daemon run";
            Restart = "on-failure";
            RestartSec = 5;
          };
        };
      };
    };

  # Home Manager: install into the user profile and register the watcher for the
  # managed user only.
  homeManager =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      cfg = config.services.josephine;
    in
    {
      options.services.josephine = {
        enable = lib.mkEnableOption "the Josephine system guardian (user daemon)";
        package = packageOption { inherit lib pkgs; };
      };

      config = lib.mkIf cfg.enable {
        home.packages = [ cfg.package ];

        systemd.user.services.josephine = {
          Unit = {
            Description = serviceDescription;
            Documentation = documentationUrl;
            After = [ "graphical-session.target" ];
          };
          Service = {
            Type = "simple";
            ExecStart = "${lib.getExe cfg.package} daemon run";
            Restart = "on-failure";
            RestartSec = 5;
          };
          Install.WantedBy = [ "default.target" ];
        };
      };
    };
}
