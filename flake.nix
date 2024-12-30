{
  description = "Rust + GTK4: wait for file, with extra button.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };


  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "wait_for_file";
          version = "0.1.0";

          # Use the current directory (which must contain Cargo.toml, Cargo.lock, src/main.rs, etc.)
          src = ./.;

          # A placeholder, so the first build will fail and tell you the correct sha256.
          # Then copy the correct value from the error message.
          cargoSha256 = "sha256-uHkydKYGaIsBSFYFxjubZXIyVU4D3g4RlKx+G43J0iw=";

          # Native build inputs for GTK4
          nativeBuildInputs = [
            pkgs.pkg-config
          ];

          # Libraries needed at build/run time
          buildInputs = [
            pkgs.gtk4
          ];

          devShell = with pkgs; mkShell {
            buildInputs = [ cargo rustc rustfmt pre-commit rustPackages.clippy pkg-config ];
          };
        };

        # For convenience, these let you do `nix run .`, `nix build .`, etc.
        defaultPackage = self.packages.${system}.default;
        defaultApp = self.packages.${system}.default;
      });
}

