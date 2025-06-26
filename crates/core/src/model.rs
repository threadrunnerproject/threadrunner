//! Core trait for any inference backend.
//!
//! This module defines the `ModelBackend` trait which provides a unified interface
//! for loading, running inference, and managing the lifecycle of language models.
//! Implementations can wrap different backends like llama.cpp or llama-rs while
//! providing a consistent API for the daemon and other components.

use anyhow::Result;
use std::path::Path;

#[cfg(feature = "dummy")]
use std::collections::VecDeque;

/// Enum for selecting backend implementation at runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    #[cfg(feature = "dummy")]
    Dummy,
    #[cfg(feature = "llama")]
    Llama,
}

/// A trait for language model inference backends.
///
/// This trait defines the core operations needed to manage a language model:
/// loading from disk, running inference with streaming token generation,
/// and proper cleanup when the model is no longer needed.
pub trait ModelBackend: Send {
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
    /// associated with the loaded model. After calling this method, the model
    /// should not be used for further operations.
    ///
    /// # Returns
    /// * `Ok(())` - Model was successfully unloaded
    /// * `Err(_)` - Error during model cleanup
    #[allow(unused_variables)]
    fn unload(&mut self) -> Result<()>;
}

/// A wrapper for boxed ModelBackend that handles cleanup automatically
/// 
/// This wrapper solves the issue where boxed trait objects can't call
/// unload(self) because trait objects can't be moved. Instead, the
/// wrapper implements Drop to handle cleanup automatically.
pub struct BoxedModelBackend {
    inner: Option<Box<dyn ModelBackend + Send>>,
}

impl BoxedModelBackend {
    pub fn new(backend: Box<dyn ModelBackend + Send>) -> Self {
        Self {
            inner: Some(backend),
        }
    }

    pub fn prompt(&mut self, text: &str) -> Result<()> {
        if let Some(ref mut backend) = self.inner {
            backend.prompt(text)
        } else {
            anyhow::bail!("Backend has been unloaded")
        }
    }

    pub fn next_token(&mut self) -> Result<Option<String>> {
        if let Some(ref mut backend) = self.inner {
            backend.next_token()
        } else {
            anyhow::bail!("Backend has been unloaded")
        }
    }

    /// Explicitly unload the backend
    pub fn unload(&mut self) -> Result<()> {
        if let Some(ref mut backend) = self.inner {
            backend.unload()?;
            self.inner = None; // Mark as unloaded
            Ok(())
        } else {
            Ok(()) // Already unloaded
        }
    }
}

impl Drop for BoxedModelBackend {
    fn drop(&mut self) {
        if let Some(ref mut backend) = self.inner {
            // Call unload on drop, but ignore errors since we're in a destructor
            let _ = backend.unload();
        }
    }
}

/// Factory function to load a backend by kind at runtime
///
/// This function provides runtime selection of backend implementations
/// based on the enabled features. Only backends that were compiled in
/// (via feature flags) will be available.
///
/// # Arguments
/// * `kind` - The type of backend to load
/// * `path` - Path to the model file
///
/// # Returns
/// * `Ok(BoxedModelBackend)` - Successfully loaded backend wrapper
/// * `Err(_)` - Error during backend loading or unsupported backend
pub fn load_backend(kind: BackendKind, path: &Path) -> Result<BoxedModelBackend> {
    let boxed_backend = match kind {
        #[cfg(feature = "dummy")]
        BackendKind::Dummy => {
            let backend = DummyBackend::load(path)?;
            Box::new(backend) as Box<dyn ModelBackend + Send>
        }
        
        #[cfg(feature = "llama")]
        BackendKind::Llama => {
            let backend = crate::llama_backend::LlamaBackend::load(path)?;
            Box::new(backend) as Box<dyn ModelBackend + Send>
        }
        
        #[cfg(not(any(feature = "dummy", feature = "llama")))]
        _ => {
            anyhow::bail!("No backends available - enable either 'dummy' or 'llama' feature")
        }
    };
    
    Ok(BoxedModelBackend::new(boxed_backend))
}

// Re-export LlamaBackend when llama feature is enabled
#[cfg(feature = "llama")]
pub use crate::llama_backend::LlamaBackend;

/// A dummy implementation of `ModelBackend` for testing and development.
///
/// This backend doesn't actually load any models but instead generates
/// lorem ipsum-style tokens for testing the inference pipeline. It's useful
/// for development and testing without requiring actual model files.
#[cfg(feature = "dummy")]
pub struct DummyBackend {
    tokens: VecDeque<String>,
}

#[cfg(feature = "dummy")]
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

    fn unload(&mut self) -> Result<()> {
        // Clear tokens to free memory
        self.tokens.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "dummy")]
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

    #[test]
    #[cfg(feature = "dummy")]
    fn factory_loads_dummy_backend() {
        let mut backend = load_backend(BackendKind::Dummy, Path::new("/dev/null")).unwrap();
        
        // Test that we can use the backend through the wrapper interface
        backend.prompt("test").unwrap();
        let token = backend.next_token().unwrap();
        assert!(token.is_some());
    }
} 