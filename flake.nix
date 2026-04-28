{
  description = "GNU Radio + RTL-SDR dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rtl-sdr

            # Rust toolchain (equivalent to languages.rust.enable = true)
            rustc
            cargo
            clippy
            rustfmt
            rust-analyzer
          ];

          # Optional: environment tweaks
          shellHook = ''
            echo "GNU Radio + RTL-SDR + Rust environment ready"
          '';
        };
      }
    );
}
