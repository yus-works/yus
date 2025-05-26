{
  description = "Dev shell for Trunk with LLD and WASM targets";

  inputs = {
    nixpkgs.url     = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = [
          pkgs.rustup             # Rustup for managing targets
          pkgs.cargo              # Cargo build tool
          pkgs.wasm-bindgen-cli   # For WASM bindings
          pkgs.wasm-pack          # Optional WASM helper
          pkgs.binaryen           # WASM optimizer (optional)
          pkgs.lld                # LLVM linker (provides rust-lld)
          pkgs.trunk-ng           # Trunk dev server & asset pipeline
        ];

        shellHook = ''
          # Ensure the WASM target is installed
          rustup target add wasm32-unknown-unknown

          echo "🔧 Ready for 'trunk serve' on ${system} (http://127.0.0.1:8080)"
        '';
      };
    });
}
