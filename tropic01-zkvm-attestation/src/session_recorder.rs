/// Session Recorder
///
/// Records the complete transcript of a TROPIC01 secure session for later
/// proof generation in the zkVM.

use serde::{Deserialize, Serialize};

/// Complete session transcript for zkVM proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTranscript {
    // Public data (will be public inputs to zkVM)
    pub chip_id: Vec<u8>,
    pub device_cert: Vec<u8>,
    pub nonce: [u8; 32],
    pub random_value: Vec<u8>,
    pub timestamp: u64,
    
    // Private witness (hidden from verifier)
    pub pairing_key: [u8; 32],
    pub l2_handshake_messages: Vec<Vec<u8>>,
    pub l3_encrypted_packets: Vec<Vec<u8>>,
    pub session_encrypt_key: [u8; 32],
    pub session_decrypt_key: [u8; 32],
    pub session_iv: u64,
}

/// Session recorder that captures all messages during L2/L3 communication
#[derive(Debug)]
pub struct SessionRecorder {
    pub l2_messages: Vec<Vec<u8>>,
    pub l3_packets: Vec<Vec<u8>>,
}

impl SessionRecorder {
    pub fn new() -> Self {
        Self {
            l2_messages: Vec::new(),
            l3_packets: Vec::new(),
        }
    }
    
    pub fn record_l2_frame(&mut self, frame: &[u8]) {
        self.l2_messages.push(frame.to_vec());
    }
    
    pub fn record_l3_packet(&mut self, packet: &[u8]) {
        self.l3_packets.push(packet.to_vec());
    }
}

impl Default for SessionRecorder {
    fn default() -> Self {
        Self::new()
    }
}
