{
  description = "Dev shell for Trunk + Tailwind + LLD + WASM targets";

  inputs = {
    nixpkgs.url     = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
      nodePackages = pkgs.nodePackages;
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = [
          # ─────────────────────────────────────────────
          # Rust + WASM toolchain
          pkgs.rustup
          pkgs.cargo
          pkgs.wasm-bindgen-cli
          pkgs.wasm-pack
          pkgs.binaryen
          pkgs.lld
          pkgs.trunk

          pkgs.openssl.dev
          pkgs.pkg-config

          # ─────────────────────────────────────────────
          # Node.js + TailwindCSS/PostCSS
          # (so that `tailwindcss` CLI, `postcss` and `autoprefixer` are in $PATH)

          pkgs.nodejs            # pulls in a recent Node 18.x
          nodePackages.tailwindcss
          nodePackages.postcss
          nodePackages.autoprefixer
        ];

        shellHook = ''
          # Make sure the WASM target is installed
          rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

          echo "⚙️  Entered dev shell on ${system}"
        '';
      };
    });
}
