pub mod session_recorder;
pub mod proof_generator;
pub mod verifier;

// Re-export commonly used types
pub use session_recorder::{SessionTranscript, SessionRecorder};
pub use proof_generator::generate_attestation_proof;
pub use verifier::verify_attestation_proof;
