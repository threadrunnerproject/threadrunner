//! IPC module for v1 framed-JSON protocol
//! 
//! This module defines the serialization types for the v1 framed-JSON IPC protocol
//! used for communication between the CLI and daemon components. The protocol
//! uses JSON messages with a version field for future compatibility.

use serde::{Serialize, Deserialize};

/// Protocol version for the framed-JSON IPC specification
pub const PROTOCOL_VERSION: u8 = 1;

/// Request structure for sending prompts to the daemon
#[derive(Serialize, Deserialize, Debug)]
pub struct PromptRequest {
    /// Protocol version
    pub v: u8,
    /// The prompt text to process
    pub prompt: String,
    /// Whether to stream the response tokens
    pub stream: bool,
}

/// Response structure for token streaming from the daemon
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    /// Optional token text (None indicates end of stream)
    pub token: Option<String>,
    /// Whether this is the end of the stream
    pub eos: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_request_serialization() {
        let request = PromptRequest {
            v: 1,
            prompt: "Hello".to_string(),
            stream: true,
        };
        
        let json = serde_json::to_string(&request).expect("Failed to serialize PromptRequest");
        
        assert!(json.contains("\"prompt\":\"Hello\""), "JSON should contain prompt field");
        assert!(json.contains("\"v\":1"), "JSON should contain version field");
    }

    #[test]
    fn test_token_response_round_trip() {
        let original = TokenResponse {
            token: Some("Hi".into()),
            eos: false,
        };
        
        let json = serde_json::to_string(&original).expect("Failed to serialize TokenResponse");
        let deserialized: TokenResponse = serde_json::from_str(&json).expect("Failed to deserialize TokenResponse");
        
        assert_eq!(original.token, deserialized.token, "Token field should match after round-trip");
        assert_eq!(original.eos, deserialized.eos, "EOS field should match after round-trip");
    }
} 