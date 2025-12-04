{
  description = "Griffin proxy";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux"; # or aarch64-darwin for macOS
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlay ];
      };

      rust = pkgs.rust-bin.stable.latest.default; 
    in
    {
      # Development shell
      devShell.${system} = pkgs.mkShell {
        buildInputs = [
          rust
          pkgs.cargo-nextest
          pkgs.pkg-config
          pkgs.openssl
        ];
      };

      # Rust package (build with `nix build`)
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "griffin";
        version = "0.0.4";
        src = ./.;

        cargoHash = pkgs.lib.fakeSha256; # replace with real hash after first build
      };

      # `nix run`
      apps.${system}.default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/griffin";
      };
    };
}
