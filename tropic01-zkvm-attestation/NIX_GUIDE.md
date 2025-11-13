# Using Nix with TROPIC01 zkVM Attestation

## Quick Start with Nix

```bash
# 1. Install Nix (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install

# 2. Enter development environment
cd libtropic-rs/tropic01-zkvm-attestation
nix develop

# 3. Install SP1 (inside Nix shell)
curl -L https://sp1.succinct.xyz | bash
source ~/.bashrc
sp1up

# 4. Build and run
./build.sh
cargo run --release -- /dev/ttyACM0 115200
```

## What the Nix Environment Provides

The `flake.nix` automatically sets up:

- ✅ **Rust 1.85.1** with RISC-V target (`riscv32im-unknown-none-elf`)
- ✅ **Build tools**: cmake, gcc, clang
- ✅ **USB/Serial libraries**: libusb, udev (for TROPIC01 communication)
- ✅ **Development tools**: cargo-watch, cargo-edit
- ✅ **Platform support**: macOS and Linux

## Why SP1 Isn't in Nix (Yet)

SP1 is currently distributed via their own installer because:

1. **Frequent updates**: SP1 is rapidly evolving, installer tracks latest versions
2. **Custom toolchain**: Includes RISC-V toolchain management
3. **Verification**: Downloads verified release artifacts
4. **Updates**: `sp1up` command for easy upgrades

The Nix environment provides **all dependencies SP1 needs**, so the installer works seamlessly.

## Using direnv (Optional but Recommended)

If you have `direnv` installed:

```bash
# Allow direnv for this directory
cd libtropic-rs/tropic01-zkvm-attestation
direnv allow

# Environment automatically loads when you cd into directory
# No need to run 'nix develop' manually!
```

## Verifying Setup

Inside the Nix development shell:

```bash
# Check Rust
rustc --version
# Should show: rustc 1.85.1

# Check RISC-V target
rustup target list | grep riscv32im
# Should show: riscv32im-unknown-none-elf (installed)

# Check SP1 (after installation)
cargo prove --version
# Should show: cargo-prove 4.0.0 (or similar)

# Check USB access
lsusb | grep -i tropic
# Should show TS1302 dongle if connected
```

## Troubleshooting

### USB Permission Issues (Linux)

If you get permission errors accessing `/dev/ttyACM0`:

```bash
# Add your user to dialout group
sudo usermod -a -G dialout $USER

# Log out and back in, or:
newgrp dialout

# Or use udev rules (inside Nix shell)
echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="0483", ATTR{idProduct}=="374b", MODE="0666"' | sudo tee /etc/udev/rules.d/99-tropic01.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### SP1 Installation Issues

If `cargo prove` isn't found after installation:

```bash
# Ensure PATH includes cargo bin
export PATH="$HOME/.cargo/bin:$PATH"

# Re-source your shell config
source ~/.bashrc  # or ~/.zshrc

# Verify installation
which cargo-prove
cargo prove --version
```

### Build Errors

If you get OpenSSL or libusb errors:

```bash
# Make sure you're in the Nix shell
nix develop

# Environment variables should be set
echo $PKG_CONFIG_PATH
echo $LIBUSB_DIR

# If still failing, try clean build
cargo clean
cargo build --release
```

## Development Workflow

Recommended workflow with Nix:

```bash
# Enter project directory (direnv loads env automatically)
cd libtropic-rs/tropic01-zkvm-attestation

# Or manually enter Nix shell
nix develop

# Build guest program (first time or after changes)
cargo prove build

# Build host program
cargo build --release

# Run with TROPIC01
cargo run --release -- /dev/ttyACM0 115200

# For development with auto-rebuild
cargo watch -x 'build --release'
```

## Updating

### Update Nix dependencies

```bash
nix flake update
```

### Update SP1

```bash
sp1up
```

### Update Rust dependencies

```bash
cargo update
```

## CI/CD Integration

Example GitHub Actions workflow with Nix:

```yaml
name: Build zkVM Attestation

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Nix
        uses: DeterminateSystems/nix-installer-action@main
      
      - name: Setup Nix cache
        uses: DeterminateSystems/magic-nix-cache-action@main
      
      - name: Build in Nix environment
        run: |
          cd libtropic-rs/tropic01-zkvm-attestation
          nix develop --command bash -c "
            curl -L https://sp1.succinct.xyz | bash
            source ~/.bashrc
            sp1up
            ./build.sh
          "
```

## Pure Nix Build (Future)

Once SP1 is packaged for Nix, you could have a fully reproducible build:

```nix
# flake.nix (future)
{
  outputs = { self, nixpkgs, sp1-nix }:
    # ...
    packages.default = pkgs.rustPlatform.buildRustPackage {
      pname = "tropic01-zkvm-attestation";
      version = "0.1.0";
      src = ./.;
      
      nativeBuildInputs = [ sp1-nix.packages.${system}.sp1 ];
      # ...
    };
}
```

For now, the hybrid approach (Nix for deps, SP1 installer for toolchain) works well.

## Benefits of Using Nix

1. **Reproducibility**: Same environment on all machines
2. **No system pollution**: Dependencies isolated to project
3. **Easy onboarding**: New developers just run `nix develop`
4. **Cross-platform**: Works on Linux, macOS, NixOS
5. **Version pinning**: Exact dependency versions locked in flake.lock

## See Also

- Nix documentation: https://nixos.org/manual/nix/stable/
- Nix flakes: https://nixos.wiki/wiki/Flakes
- direnv: https://direnv.net/
- SP1 documentation: https://docs.succinct.xyz/
