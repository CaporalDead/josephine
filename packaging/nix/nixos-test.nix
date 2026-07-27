# NixOS VM test for the module.
#
# Boots a machine that imports the NixOS module and enables the daemon, then
# checks three things:
#   1. the package is installed and runnable,
#   2. `josephine status` does not false-alarm on the read-only /nix/store
#      (the filesystem-check fix), and
#   3. the systemd *user* service is wired up and comes up for a lingering user.
{ pkgs, module }:
pkgs.testers.runNixOSTest {
  name = "josephine-module";

  nodes.machine =
    { pkgs, ... }:
    {
      imports = [ module ];

      services.josephine.enable = true;

      # A normal user whose systemd --user instance runs at boot (linger), so
      # the user service actually starts without an interactive login.
      users.users.alice = {
        isNormalUser = true;
        uid = 1000;
        linger = true;
      };

      environment.systemPackages = [ pkgs.jq ];
    };

  testScript = ''
    machine.wait_for_unit("multi-user.target")

    # 1. The package is installed and runnable.
    machine.succeed("josephine --version")

    # 2. status runs, and the filesystem check stays "ok" despite the
    #    read-only /nix/store that NixOS mounts by default.
    machine.succeed("josephine status --json > /tmp/status.json")
    machine.succeed("grep -q '/nix/store' /proc/mounts")
    machine.succeed(
        "jq -e '.[] | select(.check == \"filesystem\") | .severity == \"ok\"' /tmp/status.json"
    )

    # 3. The user daemon is enabled and reaches active for the lingering user.
    machine.wait_for_unit("user@1000.service")
    run = "su -l alice -c 'XDG_RUNTIME_DIR=/run/user/1000 systemctl --user %s josephine'"
    machine.wait_until_succeeds(run % "is-enabled")
    machine.wait_until_succeeds(run % "is-active")
  '';
}
