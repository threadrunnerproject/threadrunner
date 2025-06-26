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

/// Response structure for error information from the daemon
#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error type/kind for categorization
    pub error_type: String,
}

/// Unified response type that can be either a token or an error
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "token")]
    Token(TokenResponse),
    #[serde(rename = "error")]
    Error(ErrorResponse),
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

    #[test]
    fn test_error_response_serialization() {
        let error_response = ErrorResponse {
            error: "Model failed to load".to_string(),
            error_type: "ModelLoad".to_string(),
        };
        
        let json = serde_json::to_string(&error_response).expect("Failed to serialize ErrorResponse");
        
        assert!(json.contains("\"error\":\"Model failed to load\""), "JSON should contain error field");
        assert!(json.contains("\"error_type\":\"ModelLoad\""), "JSON should contain error_type field");
    }

    #[test]
    fn test_response_enum_serialization() {
        let token_response = Response::Token(TokenResponse {
            token: Some("hello".to_string()),
            eos: false,
        });
        
        let error_response = Response::Error(ErrorResponse {
            error: "Something went wrong".to_string(),
            error_type: "Protocol".to_string(),
        });
        
        let token_json = serde_json::to_string(&token_response).expect("Failed to serialize token response");
        let error_json = serde_json::to_string(&error_response).expect("Failed to serialize error response");
        
        assert!(token_json.contains("\"type\":\"token\""), "Token response should have type field");
        assert!(error_json.contains("\"type\":\"error\""), "Error response should have type field");
    }
} 