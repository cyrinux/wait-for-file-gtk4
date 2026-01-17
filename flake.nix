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
        commonInputs = with pkgs; [
          gtk4
          gtk3
          pkg-config
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "wait-for-file";
          version = "0.2.2";
          src = ./.;
          meta.mainProgram = "wait-for-file";
          cargoHash = "sha256-Gd2/9O8utHlcK7S+RhEy0kGtdSQHoVqv+d8Y/KTuhyI=";
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = commonInputs ++ (with pkgs; [ gdk-pixbuf ]);
        };

        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt rustPackages.clippy ]
            ++ commonInputs;
        };

        defaultPackage = self.packages.${system}.default;
        defaultApp = self.packages.${system}.default;
      });
}
