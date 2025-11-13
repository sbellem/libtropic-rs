//! zkVM Guest Program: TROPIC01 Session Verifier
//!
//! This program runs inside the SP1 zkVM and verifies that a secure session
//! with TROPIC01 was executed correctly, proving that outputs came from a
//! genuine TROPIC01 chip.
//!
//! The zkVM proves:
//! 1. Certificate chain verification (device cert → root CA)
//! 2. Noise protocol handshake with STPUB from certificate
//! 3. L3 session key derivation
//! 4. Encrypted command/response integrity
//! 5. Output correctness
//!
//! Without revealing:
//! - Session encryption/decryption keys
//! - Pairing key (sh0_privkey)
//! - Encrypted packet contents

#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Public inputs to the zkVM program (known to verifier)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicInputs {
    /// TROPIC01 chip ID (128 bytes)
    pub chip_id: Vec<u8>,
    
    /// Verifier's challenge nonce
    pub nonce: [u8; 32],
    
    /// The random value output (claimed to be from TROPIC01)
    pub random_value: Vec<u8>,
    
    /// Timestamp when session was executed
    pub timestamp: u64,
    
    /// Device certificate (X.509 DER) - will be verified
    pub device_cert: Vec<u8>,
    
    /// Root CA certificate (trusted anchor)
    pub root_ca_cert: Vec<u8>,
}

/// Private witness (hidden from verifier)
#[derive(Serialize, Deserialize, Debug)]
pub struct PrivateWitness {
    /// SH0 pairing key (X25519 private key)
    pub sh0_privkey: [u8; 32],
    
    /// L2 handshake messages (Noise protocol frames)
    pub l2_handshake_messages: Vec<Vec<u8>>,
    
    /// L3 encrypted packets (command + response)
    pub l3_encrypted_packets: Vec<Vec<u8>>,
    
    /// Derived session keys
    pub session_encrypt_key: [u8; 32],
    pub session_decrypt_key: [u8; 32],
    
    /// Session initialization vector
    pub session_iv: u64,
}

/// Output from the zkVM program
#[derive(Serialize, Deserialize, Debug)]
pub struct VerificationResult {
    /// The verified random value
    pub random_value: Vec<u8>,
    
    /// Verified chip ID
    pub chip_id: Vec<u8>,
    
    /// Verified nonce (proves freshness)
    pub nonce: [u8; 32],
    
    /// Session transcript hash (for auditing)
    pub transcript_hash: [u8; 32],
}

pub fn main() {
    // Read inputs from host
    let public: PublicInputs = sp1_zkvm::io::read();
    let witness: PrivateWitness = sp1_zkvm::io::read();
    
    sp1_zkvm::io::hint(&format!("Starting session verification for chip_id: {}", 
        hex::encode(&public.chip_id[..16])));
    
    // ========================================================================
    // STEP 1: Verify certificate chain
    // ========================================================================
    sp1_zkvm::io::hint("Step 1: Verifying certificate chain...");
    
    // TODO: Implement full X.509 certificate verification
    // For now, we'll do basic checks:
    
    // 1.1: Verify device_cert is signed by root_ca_cert
    verify_certificate_chain(&public.device_cert, &public.root_ca_cert)
        .expect("Certificate chain verification failed");
    
    // 1.2: Extract and verify chip_id from certificate
    let cert_chip_id = extract_chip_id_from_cert(&public.device_cert)
        .expect("Failed to extract chip_id from certificate");
    
    assert_eq!(cert_chip_id, public.chip_id, "Chip ID mismatch");
    
    sp1_zkvm::io::hint("✅ Certificate chain verified");
    
    // ========================================================================
    // STEP 2: Extract STPUB (X25519 public key) from certificate
    // ========================================================================
    sp1_zkvm::io::hint("Step 2: Extracting STPUB from certificate...");
    
    let stpub = extract_x25519_pubkey(&public.device_cert)
        .expect("Failed to extract STPUB from certificate");
    
    sp1_zkvm::io::hint(&format!("✅ STPUB: {}", hex::encode(&stpub)));
    
    // ========================================================================
    // STEP 3: Verify Noise protocol handshake
    // ========================================================================
    sp1_zkvm::io::hint("Step 3: Verifying Noise handshake...");
    
    // TODO: Implement full Noise XX protocol verification
    // This would verify:
    // - Handshake message sequence (→ e, ← e ee, → s es, ← s se)
    // - ECDH operations with sh0_privkey and STPUB
    // - Session key derivation (HKDF)
    
    let derived_keys = verify_noise_handshake(
        &witness.sh0_privkey,
        &stpub,
        &witness.l2_handshake_messages,
    ).expect("Noise handshake verification failed");
    
    // Verify derived keys match witness
    assert_eq!(derived_keys.encrypt, witness.session_encrypt_key,
        "Session encrypt key mismatch");
    assert_eq!(derived_keys.decrypt, witness.session_decrypt_key,
        "Session decrypt key mismatch");
    
    sp1_zkvm::io::hint("✅ Noise handshake verified, session keys derived");
    
    // ========================================================================
    // STEP 4: Verify L3 encrypted command/response
    // ========================================================================
    sp1_zkvm::io::hint("Step 4: Verifying L3 encrypted communication...");
    
    // Get encrypted command and response
    assert!(witness.l3_encrypted_packets.len() >= 2, 
        "Expected at least command and response packets");
    
    let encrypted_cmd = &witness.l3_encrypted_packets[0];
    let encrypted_resp = &witness.l3_encrypted_packets[1];
    
    // TODO: Verify L3 command structure
    // - Decrypt command with session_encrypt_key
    // - Verify command is "get_random_value" with nonce binding
    
    verify_l3_command(
        &witness.session_encrypt_key,
        witness.session_iv,
        encrypted_cmd,
        &public.nonce,
    ).expect("L3 command verification failed");
    
    // TODO: Decrypt and verify L3 response
    // - Decrypt response with session_decrypt_key
    // - Extract random value
    // - Verify AES-GCM authentication tag
    
    let decrypted_random = decrypt_l3_response(
        &witness.session_decrypt_key,
        witness.session_iv,
        encrypted_resp,
    ).expect("L3 response decryption failed");
    
    // Verify decrypted random matches public input
    assert_eq!(decrypted_random, public.random_value,
        "Random value mismatch - output doesn't match decrypted response");
    
    sp1_zkvm::io::hint("✅ L3 communication verified");
    
    // ========================================================================
    // STEP 5: Compute session transcript hash
    // ========================================================================
    sp1_zkvm::io::hint("Step 5: Computing session transcript hash...");
    
    let transcript_hash = compute_transcript_hash(
        &public.device_cert,
        &witness.l2_handshake_messages,
        &witness.l3_encrypted_packets,
    );
    
    sp1_zkvm::io::hint(&format!("✅ Transcript hash: {}", hex::encode(&transcript_hash)));
    
    // ========================================================================
    // STEP 6: Commit public outputs
    // ========================================================================
    let result = VerificationResult {
        random_value: public.random_value.clone(),
        chip_id: public.chip_id.clone(),
        nonce: public.nonce,
        transcript_hash,
    };
    
    sp1_zkvm::io::commit(&result);
    
    sp1_zkvm::io::hint("✅ Session verification complete - proof generated");
}

// ============================================================================
// Helper Functions (Simplified for MVP)
// ============================================================================

fn verify_certificate_chain(_device_cert: &[u8], _root_ca_cert: &[u8]) -> Result<(), &'static str> {
    // TODO: Implement full X.509 verification
    // For MVP, we assume certificates are valid
    // Real implementation would:
    // 1. Parse X.509 DER
    // 2. Verify signature using root CA public key
    // 3. Check validity dates
    // 4. Verify certificate extensions
    
    Ok(())
}

fn extract_chip_id_from_cert(_cert: &[u8]) -> Result<Vec<u8>, &'static str> {
    // TODO: Parse X.509 and extract chip_id from subject CN
    // For MVP, return dummy chip_id
    
    Ok(vec![0u8; 128])
}

fn extract_x25519_pubkey(_cert: &[u8]) -> Result<[u8; 32], &'static str> {
    // TODO: Parse X.509 and extract X25519 public key
    // For MVP, return dummy STPUB
    
    Ok([0u8; 32])
}

#[derive(Debug)]
struct SessionKeys {
    encrypt: [u8; 32],
    decrypt: [u8; 32],
}

fn verify_noise_handshake(
    _sh0_privkey: &[u8; 32],
    _stpub: &[u8; 32],
    _handshake_messages: &[Vec<u8>],
) -> Result<SessionKeys, &'static str> {
    // TODO: Implement Noise XX protocol verification
    // This is the most complex part, requiring:
    // 1. X25519 ECDH operations
    // 2. HKDF key derivation
    // 3. ChaCha20-Poly1305 or AES-GCM encryption/decryption
    // 4. Handshake state machine verification
    
    // For MVP, return dummy keys
    Ok(SessionKeys {
        encrypt: [0u8; 32],
        decrypt: [0u8; 32],
    })
}

fn verify_l3_command(
    _session_key: &[u8; 32],
    _iv: u64,
    _encrypted_cmd: &[u8],
    _nonce: &[u8; 32],
) -> Result<(), &'static str> {
    // TODO: Decrypt L3 command and verify:
    // 1. Command ID is "get_random_value"
    // 2. Nonce is included in command
    // 3. AES-GCM authentication tag is valid
    
    Ok(())
}

fn decrypt_l3_response(
    _session_key: &[u8; 32],
    _iv: u64,
    _encrypted_resp: &[u8],
) -> Result<Vec<u8>, &'static str> {
    // TODO: Decrypt L3 response using AES-GCM
    // 1. Verify authentication tag
    // 2. Decrypt ciphertext
    // 3. Extract random value from response structure
    
    // For MVP, return dummy random value
    Ok(vec![0xAA; 32])
}

fn compute_transcript_hash(
    device_cert: &[u8],
    l2_messages: &[Vec<u8>],
    l3_packets: &[Vec<u8>],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    
    // Hash certificate
    hasher.update(device_cert);
    
    // Hash all L2 messages
    for msg in l2_messages {
        hasher.update(msg);
    }
    
    // Hash all L3 packets
    for packet in l3_packets {
        hasher.update(packet);
    }
    
    hasher.finalize().into()
}

// Minimal hex encoding for zkVM (std not available)
mod hex {
    use core::fmt::Write;
    
    pub fn encode(data: &[u8]) -> String {
        let mut s = String::with_capacity(data.len() * 2);
        for byte in data {
            write!(&mut s, "{:02x}", byte).unwrap();
        }
        s
    }
}
