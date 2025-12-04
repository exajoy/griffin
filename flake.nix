{
  description = "Griffin - Rust gRPC/gRPC-Web proxy workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    crane.url = "github:ipetkov/crane";

    rust-advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    { self, nixpkgs, rust-overlay, flake-utils, crane, rust-advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Read version from rust-toolchain.toml
        rustToolchainToml = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
        rustVersion = rustToolchainToml.toolchain.channel;

        rustToolchain = pkgs.rust-bin.stable.${rustVersion}.default.override {
          extensions = ["rust-src" "rustfmt" "clippy"];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Ensure source filtering includes your crates
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            let
              rel = pkgs.lib.removePrefix (toString ./. + "/") (toString path);
            in
              craneLib.filterCargoSources path type
              || (pkgs.lib.hasPrefix "griffin/" rel);
              # || (pkgs.lib.hasPrefix "griffin-core/" rel)
              # || (pkgs.lib.hasPrefix "griffin-test/" rel);
              # || (pkgs.lib.hasPrefix "proto/" rel);  # remove if unused
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          nativeBuildInputs = with pkgs; [
            pkg-config
            protobuf
            cmake
          ];

          buildInputs =
            with pkgs;
            [
              openssl
              sqlite
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              apple-sdk_26
            ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build individual crates
        mkCrate =
          { name, cargoPackage ? name }:
          craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            pname = name;
            cargoExtraArgs = "-p ${cargoPackage}";
          });

        devTools = with pkgs; [
          cargo-watch
          cargo-edit
          cargo-outdated
          cargo-audit
          cargo-nextest
          just
          bacon
          nushell
          rust-analyzer
          python3
          nodejs
        ];

      in
      {
        # -----------------------
        # Packages (nix build)
        # -----------------------
        packages = {
          default = self.packages.${system}.griffin;

          griffin = mkCrate { name = "griffin"; };
          griffin-core = mkCrate { name = "griffin-core"; };
          griffin-test = mkCrate { name = "griffin-test"; };

          # Build everything together
          griffin-workspace = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            pname = "griffin-workspace";
          });
        };

        # -----------------------
        # Dev Shell
        # -----------------------
        devShells.default = pkgs.mkShell {
          inherit (commonArgs) buildInputs;
          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ rustToolchain ] ++ devTools;

          shellHook = ''
            echo "Griffin dev shell loaded."
          '';

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          RUST_BACKTRACE = 1;

          PKG_CONFIG_PATH = pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" [ pkgs.openssl ];
        };

        # -----------------------
        # Apps (nix run)
        # -----------------------
        apps = {
          default = self.apps.${system}.griffin;

          griffin = {
            type = "app";
            program = "${self.packages.${system}.griffin}/bin/griffin";
          };
        };

        # -----------------------
        # Checks (CI)
        # -----------------------
        checks = {
          griffin-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });

          griffin-fmt = craneLib.cargoFmt { src = craneLib.cleanCargoSource ./.; };

          griffin-tests = craneLib.cargoTest (commonArgs // { inherit cargoArtifacts; });

          griffin-audit = craneLib.cargoAudit {
            src = craneLib.cleanCargoSource ./.;
            advisory-db = rust-advisory-db;
            cargoAuditExtraArgs = "--ignore RUSTSEC-2023-0071";
          };

          griffin-doc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

          griffin-build = self.packages.${system}.griffin-workspace;
        };
      }
    );
}
