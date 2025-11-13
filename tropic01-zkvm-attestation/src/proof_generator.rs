/// Proof Generator
///
/// Generates zkVM proofs for TROPIC01 session transcripts

use crate::session_recorder::SessionTranscript;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sp1_sdk::{ProverClient, SP1Stdin};

/// Include the compiled zkVM guest program
pub const SESSION_VERIFIER_ELF: &[u8] = include_bytes!("../../elf/riscv32im-succinct-zkvm-elf");

/// Public inputs for zkVM (visible to verifier)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicInputs {
    pub chip_id: Vec<u8>,
    pub nonce: [u8; 32],
    pub random_value: Vec<u8>,
    pub timestamp: u64,
    pub device_cert: Vec<u8>,
    pub root_ca_cert: Vec<u8>,
}

/// Private witness for zkVM (hidden from verifier)
#[derive(Serialize, Deserialize, Debug)]
pub struct PrivateWitness {
    pub sh0_privkey: [u8; 32],
    pub l2_handshake_messages: Vec<Vec<u8>>,
    pub l3_encrypted_packets: Vec<Vec<u8>>,
    pub session_encrypt_key: [u8; 32],
    pub session_decrypt_key: [u8; 32],
    pub session_iv: u64,
}

/// Attestation proof (sent to verifier)
#[derive(Serialize, Deserialize, Debug)]
pub struct AttestationProof {
    pub chip_id: Vec<u8>,
    pub device_cert: Vec<u8>,
    pub random_value: Vec<u8>,
    pub nonce: [u8; 32],
    pub timestamp: u64,
    
    /// SP1 proof (can be verified by anyone)
    pub sp1_proof: Vec<u8>,
    
    /// Public values committed by the zkVM program
    pub public_values: Vec<u8>,
}

/// Generate attestation proof using SP1 zkVM
pub fn generate_attestation_proof(
    transcript: &SessionTranscript,
    root_ca_cert: Vec<u8>,
) -> Result<AttestationProof> {
    println!("════════════════════════════════════════════════════════");
    println!("Generating zkVM Attestation Proof");
    println!("════════════════════════════════════════════════════════\n");
    
    // Prepare public inputs
    let public_inputs = PublicInputs {
        chip_id: transcript.chip_id.clone(),
        nonce: transcript.nonce,
        random_value: transcript.random_value.clone(),
        timestamp: transcript.timestamp,
        device_cert: transcript.device_cert.clone(),
        root_ca_cert: root_ca_cert.clone(),
    };
    
    println!("Public inputs:");
    println!("  Chip ID: {}...", hex::encode(&public_inputs.chip_id[..16]));
    println!("  Nonce: {}", hex::encode(&public_inputs.nonce));
    println!("  Random: {}...", hex::encode(&public_inputs.random_value[..16]));
    println!("  Timestamp: {}", public_inputs.timestamp);
    println!();
    
    // Prepare private witness
    let witness = PrivateWitness {
        sh0_privkey: transcript.pairing_key,
        l2_handshake_messages: transcript.l2_handshake_messages.clone(),
        l3_encrypted_packets: transcript.l3_encrypted_packets.clone(),
        session_encrypt_key: transcript.session_encrypt_key,
        session_decrypt_key: transcript.session_decrypt_key,
        session_iv: transcript.session_iv,
    };
    
    println!("Private witness:");
    println!("  L2 messages: {}", witness.l2_handshake_messages.len());
    println!("  L3 packets: {}", witness.l3_encrypted_packets.len());
    println!("  (Keys and secrets hidden from verifier)");
    println!();
    
    // Create SP1 prover client
    let client = ProverClient::new();
    
    // Prepare stdin for zkVM
    let mut stdin = SP1Stdin::new();
    stdin.write(&public_inputs);
    stdin.write(&witness);
    
    println!("Executing program in zkVM...");
    println!("(This may take 30 seconds to 5 minutes depending on your hardware)");
    
    // Generate proof
    let (public_values, proof) = client.prove(SESSION_VERIFIER_ELF, stdin)
        .run()
        .map_err(|e| anyhow::anyhow!("zkVM proof generation failed: {}", e))?;
    
    println!("\n✅ Proof generated successfully!");
    println!("   Proof size: {} bytes", proof.bytes().len());
    println!("   Public values size: {} bytes", public_values.as_slice().len());
    
    Ok(AttestationProof {
        chip_id: transcript.chip_id.clone(),
        device_cert: transcript.device_cert.clone(),
        random_value: transcript.random_value.clone(),
        nonce: transcript.nonce,
        timestamp: transcript.timestamp,
        sp1_proof: proof.bytes().to_vec(),
        public_values: public_values.as_slice().to_vec(),
    })
}
