# TROPIC01 zkVM Remote Attestation (MVP)

This project demonstrates **zero-knowledge proof-based remote attestation** for TROPIC01, enabling remote verifiers to cryptographically confirm that outputs came from a genuine TROPIC01 chip via a valid secure session.

## The Problem

TROPIC01's certificate only contains an **X25519 key** (for Diffie-Hellman key exchange in Noise protocol). This key:
- ✅ Enables secure L3 sessions (authenticated encryption)
- ❌ **Cannot sign outputs** (X25519 is ECDH-only, not a signature scheme)

**Result**: Host MCU can verify outputs (via encrypted session), but **remote verifiers cannot**.

## The Solution: zkVM

Run the entire secure session inside a **zero-knowledge virtual machine** (zkVM):

```
┌──────────────────────────────────────────────────────┐
│  Host MCU                                            │
│  1. Execute real session with TROPIC01               │
│  2. Record complete transcript (L2/L3 messages)      │
│  3. Send transcript to zkVM prover                   │
└──────────────────────────────────────────────────────┘
                    ↓ Session transcript
┌──────────────────────────────────────────────────────┐
│  zkVM Prover (SP1)                                   │
│  • Verifies certificate chain                        │
│  • Replays Noise handshake with STPUB from cert      │
│  • Verifies L3 encryption/decryption                 │
│  • Outputs: random_value + proof π                   │
│  Time: ~1-5 minutes                                  │
└──────────────────────────────────────────────────────┘
                    ↓ Proof π (~100-200 KB)
┌──────────────────────────────────────────────────────┐
│  Remote Verifier                                     │
│  • Receives: random_value, chip_id, cert, proof π    │
│  • Verifies: proof π in ~10-100ms                    │
│  • Trusts: Output came from genuine TROPIC01         │
└──────────────────────────────────────────────────────┘
```

## What Gets Proven

**Public** (known to verifier):
- ✅ TROPIC01 chip_id
- ✅ Device certificate (X.509)
- ✅ Random value output
- ✅ Verifier's nonce
- ✅ Timestamp

**Private** (hidden from verifier):
- 🔒 Session encryption/decryption keys
- 🔒 Pairing key (sh0_privkey)
- 🔒 L2/L3 encrypted packet contents

**Proof statement**:
> "There exists a valid execution where the random_value was obtained from a secure L3 session with TROPIC01 (certificate X, chaining to Tropic Square CA), without revealing session keys."

## Architecture

### 1. Host Program (`src/main.rs`)
- Executes real session with TROPIC01
- Records transcript (L2 handshake, L3 packets)
- Calls proof generator

### 2. Session Recorder (`src/session_recorder.rs`)
- Captures all L2/L3 messages
- Stores public/private data separately

### 3. Proof Generator (`src/proof_generator.rs`)
- Prepares inputs for zkVM
- Runs SP1 prover
- Returns attestation proof

### 4. zkVM Guest Program (`methods/guest/src/main.rs`)
- Runs inside SP1 zkVM (isolated, provable)
- Verifies:
  1. Certificate chain
  2. Noise protocol handshake
  3. L3 encryption/decryption
  4. Output correctness
- Commits public outputs

### 5. Verifier (`src/verifier.rs`)
- Checks SP1 proof validity
- Verifies certificate chain
- Confirms outputs match claims

## Building

### Prerequisites

#### Option A: Using Nix (Recommended)

1. **Install Nix** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
   ```

2. **Enter development environment**:
   ```bash
   cd libtropic-rs/tropic01-zkvm-attestation
   nix develop
   ```
   
   This automatically provides:
   - Rust 1.85.1 with RISC-V target
   - All build dependencies (cmake, gcc, clang)
   - USB/serial libraries for TROPIC01
   - Development tools

3. **Install SP1 toolchain** (inside Nix shell):
   ```bash
   curl -L https://sp1.succinct.xyz | bash
   source ~/.bashrc  # or restart shell
   sp1up
   cargo prove --version
   ```

   **Note**: SP1 is currently installed via their installer (not Nix package) as it includes
   custom toolchain management. The Nix environment provides all dependencies SP1 needs.

#### Option B: Manual Installation

1. **Install Rust** (1.85.1+):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add riscv32im-unknown-none-elf
   ```

2. **Install SP1 toolchain**:
   ```bash
   curl -L https://sp1.succinct.xyz | bash
   sp1up
   cargo prove --version
   ```

3. **Install system dependencies**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install pkg-config libusb-1.0-0-dev libudev-dev cmake gcc clang
   
   # Fedora
   sudo dnf install pkg-config libusb-devel systemd-devel cmake gcc clang
   
   # macOS
   brew install pkg-config libusb cmake
   ```

#### Hardware Requirements
   - TROPIC01 chip with TS1302 USB dongle
   - ~4-8 GB RAM for proof generation
   - ~10-20 GB disk space for SP1 toolchain

### Build

```bash
cd libtropic-rs/tropic01-zkvm-attestation

# Build the zkVM guest program
cargo prove build

# Build the host program
cargo build --release
```

## Running

```bash
# Connect TROPIC01 via TS1302 dongle
cargo run --release -- /dev/ttyACM0 115200
```

### Expected Output

```
════════════════════════════════════════════════════════
TROPIC01 zkVM Remote Attestation Example
════════════════════════════════════════════════════════

This example demonstrates:
  1. Capturing TROPIC01 session transcript
  2. Generating zkVM proof of valid session
  3. Verifying proof (remote verifier simulation)

════════════════════════════════════════════════════════
Phase 1: Execute Session with TROPIC01
════════════════════════════════════════════════════════

Opening TS1302 dongle on /dev/ttyACM0 @ 115200 baud
Loading SH0 pairing key...
Verifier challenge nonce: 9f8e7d6c...
Starting secure session with TROPIC01...
✅ Secure session established

Reading chip ID...
  Chip ID: 0123456789abcdef...
Getting random value from TROPIC01...
  Random: a1b2c3d4e5f6...

✅ Session transcript captured
   L2 messages: 4
   L3 packets: 2
   Saved to: session_transcript.json

════════════════════════════════════════════════════════
Phase 2: Generate zkVM Proof
════════════════════════════════════════════════════════

Generating proof (this may take 30 seconds to 5 minutes)...

Public inputs:
  Chip ID: 0123456789abcdef...
  Nonce: 9f8e7d6c...
  Random: a1b2c3d4...
  Timestamp: 1731484800

Private witness:
  L2 messages: 4
  L3 packets: 2
  (Keys and secrets hidden from verifier)

Executing program in zkVM...
(This may take 30 seconds to 5 minutes depending on your hardware)

✅ Proof generated successfully!
   Proof size: 142856 bytes
   Public values size: 328 bytes
   Saved to: attestation_proof.json

════════════════════════════════════════════════════════
Phase 3: Verify Proof (Remote Verifier)
════════════════════════════════════════════════════════

Simulating remote verifier receiving proof...

Proof metadata:
  Chip ID: 0123456789abcdef...
  Random value: a1b2c3d4...
  Nonce: 9f8e7d6c...
  Timestamp: 1731484800
  Proof size: 142856 bytes

✅ Timestamp fresh (3 seconds old)
✅ Certificate chain valid (simulated)

Verifying zkVM proof...
(This should take ~10-100ms)
✅ zkVM proof verified successfully!

zkVM program outputs:
  Random value: a1b2c3d4...
  Chip ID: 0123456789abcdef...
  Nonce: 9f8e7d6c...
  Transcript hash: f1e2d3c4b5a6...

✅ All checks passed!

════════════════════════════════════════════════════════
✅ ATTESTATION VERIFIED
════════════════════════════════════════════════════════

Verifier is convinced that:
  • Random value came from genuine TROPIC01 chip
  • Certificate chain is valid (chains to Tropic Square CA)
  • Secure session was executed correctly (Noise protocol)
  • L3 encryption/decryption was performed correctly
  • Proof is fresh (nonce matches, timestamp recent)
  • Host could NOT have forged this proof

Without learning:
  • Session encryption/decryption keys
  • Pairing key (sh0_privkey)
  • Encrypted packet contents
```

## MVP Status

This is a **Minimum Viable Product** demonstrating the architecture. The following are simplified/stubbed:

### ✅ Implemented
- SP1 zkVM integration
- Session transcript recording structure
- Proof generation workflow
- Proof verification workflow
- End-to-end example

### ⚠️ Simplified (MVP)
- Certificate verification (stubbed)
- Noise protocol replay (stubbed)
- AES-GCM encryption/decryption (stubbed)
- L2/L3 packet capture (placeholders)

### 📋 TODO for Production

1. **Implement Noise Protocol in zkVM**
   - X25519 ECDH operations
   - ChaCha20-Poly1305 or AES-GCM AEAD
   - HKDF key derivation
   - Handshake state machine

2. **Implement X.509 Certificate Parsing**
   - DER decoding in zkVM
   - Signature verification (ECDSA/Ed25519)
   - Certificate chain validation
   - Extension parsing (extract STPUB)

3. **Implement AES-GCM in zkVM**
   - Encryption/decryption
   - Tag verification
   - Support for L3 packet format

4. **Modify libtropic for Transcript Capture**
   - Add hooks to record L2 frames
   - Add hooks to record L3 packets
   - Export session keys (securely, only for proof generation)
   - Minimal API changes

5. **Optimize Performance**
   - Parallelize proof generation
   - Use lookup tables where possible
   - Minimize circuit complexity

6. **Security Audit**
   - Review zkVM implementation
   - Verify all crypto primitives
   - Test attack scenarios

## Performance

Expected performance (estimated):

| Operation | Time | Size |
|-----------|------|------|
| Session execution | ~1-2 seconds | - |
| Proof generation | ~1-5 minutes | ~100-200 KB |
| Proof verification | ~10-100 ms | - |
| Network transmission | ~100-500 ms | ~100-200 KB |

Total latency: **~1-5 minutes** (dominated by proof generation)

This is acceptable for attestation use cases (done once or infrequently).

## Advantages

✅ **No firmware changes required** - Works with current TROPIC01
✅ **No TEE required** - Standard hardware
✅ **Trustless verification** - Cryptographic proof, not trust in host/TEE
✅ **Privacy preserving** - Session keys stay private
✅ **Flexible** - Can prove complex session flows
✅ **Future-proof** - Can upgrade zkVM without changing TROPIC01

## Limitations

⚠️ **Proving time** - 1-5 minutes (not real-time)
⚠️ **Proof size** - 100-200 KB (larger than signatures)
⚠️ **Implementation complexity** - Requires crypto in zkVM
⚠️ **Computational cost** - Significant CPU for proving

## Comparison with Alternatives

| Approach | Changes Required | Proving Time | Trust Model |
|----------|-----------------|--------------|-------------|
| **zkVM (this)** | None | ~1-5 min | Trustless |
| TEE-based | None | ~1 sec | Trust TEE |
| L2 ATTEST | Firmware | ~1 sec | Trust bootloader |
| Dual-key cert | Manufacturing | ~1 ms | Trust chip |

## Use Cases

- Remote device attestation
- Supply chain verification
- Cloud service integration
- Compliance/audit logging
- Multi-party computation with TROPIC01

## License

See root `LICENSE` file.

## Contributing

For production implementation, contributions needed for:
- Noise protocol in zkVM
- X.509 parsing in zkVM
- AES-GCM in zkVM
- libtropic transcript capture hooks

See `ZKVM_REMOTE_ATTESTATION.md` for detailed design.
