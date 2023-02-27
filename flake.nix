{
  # Flake inputs
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  # Flake outputs
  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    # Overlays enable you to customize the Nixpkgs attribute set
    overlays = [
      # Makes a `rust-bin` attribute available in Nixpkgs
      (import rust-overlay)
      # Provides a `rustToolchain` attribute for Nixpkgs that we can use to
      # create a Rust environment
      (self: super: {
        rustToolchain = super.rust-bin.stable.latest.default;
      })
      (self: super: {
        flatbuffers = super.flatbuffers.overrideAttrs (final: prev: rec {
          version = "23.1.21";
          src = super.fetchFromGitHub {
            owner = "google";
            repo = "flatbuffers";
            rev = "v${version}";
            sha256 = "sha256-/46Yo186PjewYN+e/UWZc0QQhXZcq/x7iaN48RA1avw=";
          };
        });
      })
    ];

    # Helper to provide system-specific attributes
    allSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = f:
      nixpkgs.lib.genAttrs allSystems (system:
        f {
          pkgs = import nixpkgs {inherit overlays system;};
        });
  in {
    # Development environment output
    devShells = forAllSystems ({pkgs}: {
      default = pkgs.mkShell {
        # The Nix packages provided in the environment
        packages =
          (with pkgs; [
            # The package provided by our custom overlay. Includes cargo, Clippy, cargo-fmt,
            # rustdoc, rustfmt, and other tools.
            rustToolchain
            nodejs-18_x
            yarn
            flatbuffers
          ])
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [libiconv]);
      };
    });
  };
}
