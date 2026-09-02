# Build the josephine binary from the workspace source.
#
# Version is read straight from the workspace Cargo.toml, and dependencies are
# vendored from the committed Cargo.lock, so nothing here needs bumping on a new
# release: tag the repo and the flake follows.
{
  lib,
  rustPlatform,
  # Not named `src`: callPackage would then auto-fill it from `pkgs.src` (a
  # deprecated throwing alias). The default resolves to the repository root, so
  # `callPackage ./package.nix { }` just works; the flake source is git-tracked
  # only, so no manual filtering is needed there.
  source ? ../..,
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../../Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = "josephine";
  version = cargoToml.workspace.package.version;

  src = source;

  cargoLock.lockFile = ../../Cargo.lock;

  # The repository root is a virtual workspace (josephine-core + josephine); aim
  # cargo at the binary crate so we build and install just the CLI.
  cargoBuildFlags = [
    "-p"
    "josephine"
  ];

  # The system checks (systemd, /sys, ping, journald) are not reachable inside
  # the Nix build sandbox and degrade to "unavailable" there, so the CLI
  # integration tests cannot exercise them meaningfully. Run the pure library
  # tests instead, matching how the project's own CI keeps env-dependent checks
  # out of the test run.
  cargoTestFlags = [
    "-p"
    "josephine-core"
  ];

  meta = {
    description = "Your computer's quiet guardian angel - a local Linux system watcher";
    homepage = "https://github.com/systm-d/josephine";
    changelog = "https://github.com/systm-d/josephine/blob/v${cargoToml.workspace.package.version}/CHANGELOG.md";
    license = with lib.licenses; [
      mit
      asl20
    ];
    platforms = lib.platforms.linux;
    mainProgram = "josephine";
  };
}
