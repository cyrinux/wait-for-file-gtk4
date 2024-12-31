{
  description = "GTK4 app that wait for file, start a command, with extra button.";

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
          pname = "wait-for-file";
          version = "0.1.0";

          src = ./.;

          cargoHash = "sha256-+1ouLQpGqZbMFxiQ7k+zh4uii4nbVcP9H1qRcSOGmW8=";

          # Native build inputs for GTK4
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          # Libraries needed at build/run time
          buildInputs = with pkgs; [
            gtk4
            gtk3
            gdk-pixbuf
          ];
        };

        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt pre-commit rustPackages.clippy pkg-config gtk4 gtk3 ];
        };

        # For convenience, these let you do `nix run .`, `nix build .`, etc.
        defaultPackage = self.packages.${system}.default;
        defaultApp = self.packages.${system}.default;
      });
}

