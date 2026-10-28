{
  description = "markovpass — Markov chain based passphrase generator";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (
        pkgs:
        let
          cargo = pkgs.lib.importTOML ./Cargo.toml;
          markovpass = pkgs.rustPlatform.buildRustPackage {
            pname = cargo.package.name;
            version = cargo.package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            # Compile the packaged corpus location into the binary. It is a
            # fallback after the platform data directory and XDG_DATA_DIRS.
            env.MARKOVPASS_DEFAULT_CORPUS_DIR = "${placeholder "out"}/share/markovpass";

            postInstall = ''
              mkdir -p $out/share/markovpass
              install -m644 pkg/*.txt $out/share/markovpass/
            '';

            meta = {
              description = cargo.package.description;
              homepage = cargo.package.repository;
              license = pkgs.lib.licenses.bsd3;
              mainProgram = "markovpass";
              platforms = pkgs.lib.platforms.unix;
            };
          };
        in
        {
          inherit markovpass;
          default = markovpass;
        }
      );

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          name = "markovpass";
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
          ];

          # `cargo run` in the shell uses this flake's bundled corpus.
          MARKOVPASS_DEFAULT_CORPUS_DIR = "${./pkg}";
        };
      });
    };
}
