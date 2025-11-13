{
  description = "TROPIC01 zkVM attestation with SP1";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Rust toolchain matching workspace requirements
        rustToolchain = pkgs.rust-bin.stable."1.85.1".default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
        
        # SP1 requires RISC-V target
        rustToolchainWithRiscV = rustToolchain.override {
          targets = [ "riscv32im-unknown-none-elf" ];
        };
        
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain with RISC-V support
            rustToolchainWithRiscV
            
            # Build dependencies
            pkg-config
            openssl
            
            # SP1 dependencies
            cmake
            gcc
            clang
            
            # USB/Serial for TROPIC01
            libusb1
            udev
            
            # Development tools
            cargo-watch
            cargo-edit
            just
            
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
            libiconv
          ];

          shellHook = ''
            echo "═══════════════════════════════════════════════════"
            echo "TROPIC01 zkVM Attestation Development Environment"
            echo "═══════════════════════════════════════════════════"
            echo ""
            echo "Rust: $(rustc --version)"
            echo "Cargo: $(cargo --version)"
            echo ""
            
            # Check if SP1 is installed
            if command -v cargo-prove &> /dev/null; then
              echo "✅ SP1 installed: $(cargo prove --version)"
            else
              echo "⚠️  SP1 not installed"
              echo ""
              echo "To install SP1:"
              echo "  curl -L https://sp1.succinct.xyz | bash"
              echo "  source ~/.bashrc  # or restart shell"
              echo "  sp1up"
              echo ""
              echo "Note: SP1 installation is currently done via their installer,"
              echo "      not through Nix, as it includes toolchain management."
            fi
            
            echo ""
            echo "To build:"
            echo "  ./build.sh"
            echo ""
            echo "To run:"
            echo "  cargo run --release -- /dev/ttyACM0 115200"
            echo ""
          '';
          
          # Set environment variables for USB access
          LIBUSB_DIR = "${pkgs.libusb1}";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          
          # Prevent cargo from using system SSL
          CARGO_BUILD_TARGET_DIR = "target";
        };
      }
    );
}
