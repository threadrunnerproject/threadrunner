//! Core trait for any inference backend.
//!
//! This module defines the `ModelBackend` trait which provides a unified interface
//! for loading, running inference, and managing the lifecycle of language models.
//! Implementations can wrap different backends like llama.cpp or llama-rs while
//! providing a consistent API for the daemon and other components.

use anyhow::Result;
use std::collections::VecDeque;
use std::path::Path;

/// A trait for language model inference backends.
///
/// This trait defines the core operations needed to manage a language model:
/// loading from disk, running inference with streaming token generation,
/// and proper cleanup when the model is no longer needed.
pub trait ModelBackend {
    /// Load a model from the specified path.
    ///
    /// This method should load the model file (typically a GGUF file) into memory
    /// and prepare it for inference. The implementation should use memory mapping
    /// when possible for efficient loading of large model files.
    ///
    /// # Arguments
    /// * `model_path` - Path to the model file to load
    ///
    /// # Returns
    /// * `Ok(Self)` - Successfully loaded model backend
    /// * `Err(_)` - Error during model loading
    #[allow(unused_variables)]
    fn load(model_path: &Path) -> Result<Self>
    where
        Self: Sized;

    /// Submit a prompt for inference.
    ///
    /// This method processes the input text and prepares the model for token generation.
    /// After calling this method, tokens can be retrieved one at a time using `next_token()`.
    ///
    /// # Arguments
    /// * `text` - The input prompt text to process
    ///
    /// # Returns
    /// * `Ok(())` - Prompt was successfully processed
    /// * `Err(_)` - Error during prompt processing
    #[allow(unused_variables)]
    fn prompt(&mut self, text: &str) -> Result<()>;

    /// Generate the next token from the current inference session.
    ///
    /// This method should be called repeatedly after `prompt()` to retrieve
    /// generated tokens one at a time for streaming output. Returns `None`
    /// when the model has finished generating (reached end-of-sequence).
    ///
    /// # Returns
    /// * `Ok(Some(token))` - Next generated token
    /// * `Ok(None)` - Generation is complete
    /// * `Err(_)` - Error during token generation
    #[allow(unused_variables)]
    fn next_token(&mut self) -> Result<Option<String>>;

    /// Unload the model and free associated resources.
    ///
    /// This method should clean up any memory, file handles, or other resources
    /// associated with the loaded model. It consumes `self` to ensure the model
    /// cannot be used after unloading.
    ///
    /// # Returns
    /// * `Ok(())` - Model was successfully unloaded
    /// * `Err(_)` - Error during model cleanup
    #[allow(unused_variables)]
    fn unload(self) -> Result<()>;
}

/// A dummy implementation of `ModelBackend` for testing and development.
///
/// This backend doesn't actually load any models but instead generates
/// lorem ipsum-style tokens for testing the inference pipeline. It's useful
/// for development and testing without requiring actual model files.
pub struct DummyBackend {
    tokens: VecDeque<String>,
}

impl ModelBackend for DummyBackend {
    fn load(_model_path: &Path) -> Result<Self> {
        // Seed with some lorem ipsum words
        let lorem_words = vec![
            "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit",
            "sed", "do", "eiusmod", "tempor", "incididunt", "ut", "labore", "et", "dolore",
            "magna", "aliqua", "enim", "ad", "minim", "veniam", "quis", "nostrud",
        ];
        
        let tokens = lorem_words.into_iter().map(String::from).collect();
        
        Ok(DummyBackend { tokens })
    }

    fn prompt(&mut self, text: &str) -> Result<()> {
        // Split the input text by whitespace and append as "model-ready" tokens
        let input_tokens: Vec<String> = text
            .split_whitespace()
            .map(|word| format!("{}.", word)) // Add a period to simulate processing
            .collect();
        
        self.tokens.extend(input_tokens);
        Ok(())
    }

    fn next_token(&mut self) -> Result<Option<String>> {
        // Pop from the front and return the token
        Ok(self.tokens.pop_front())
    }

    fn unload(self) -> Result<()> {
        // Simply drop self - no cleanup needed for dummy backend
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_load_and_stream() {
        // Load the dummy backend
        let mut backend = DummyBackend::load(Path::new("/dev/null")).unwrap();
        
        // Send a prompt
        backend.prompt("lorem ipsum dolor sit amet").unwrap();
        
        // Collect all tokens
        let mut tokens = Vec::new();
        while let Some(token) = backend.next_token().unwrap() {
            tokens.push(token);
        }
        
        // Assert that we get the seeded lorem words plus our prompt tokens
        // The backend starts with 25 lorem words, then adds 5 processed prompt words
        assert_eq!(tokens.len(), 30);
        
        // Check that the first few tokens are from the seeded lorem words
        assert_eq!(tokens[0], "lorem");
        assert_eq!(tokens[1], "ipsum");
        assert_eq!(tokens[2], "dolor");
        
        // Check that the last 5 tokens are our processed prompt words
        let prompt_tokens = &tokens[25..30];
        assert_eq!(prompt_tokens[0], "lorem.");
        assert_eq!(prompt_tokens[1], "ipsum.");
        assert_eq!(prompt_tokens[2], "dolor.");
        assert_eq!(prompt_tokens[3], "sit.");
        assert_eq!(prompt_tokens[4], "amet.");
        
        // Verify that the next call returns None
        assert_eq!(backend.next_token().unwrap(), None);
    }
} 