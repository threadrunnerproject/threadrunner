use anyhow::Result;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

#[cfg(feature = "llama")]
use llama_cpp::{LlamaModel, LlamaParams, LlamaSession, SessionParams};
#[cfg(feature = "llama")]
use llama_cpp::standard_sampler::StandardSampler;

use crate::model::ModelBackend;

#[cfg(feature = "llama")]
pub struct LlamaBackend {
    model: LlamaModel,
    session: Option<LlamaSession>,
    token_receiver: Option<Receiver<Option<String>>>,
    worker_handle: Option<JoinHandle<()>>,
    stop_sender: Option<Sender<()>>,
}

#[cfg(feature = "llama")]
impl LlamaBackend {
    pub fn new(model: LlamaModel) -> Self {
        Self {
            model,
            session: None,
            token_receiver: None,
            worker_handle: None,
            stop_sender: None,
        }
    }

    fn stop_generation(&mut self) {
        // Signal the worker thread to stop
        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }
        
        // Wait for the worker thread to finish
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
        
        // Clear the receiver
        self.token_receiver = None;
    }
}

#[cfg(feature = "llama")]
impl ModelBackend for LlamaBackend {
    fn load(model_path: &Path) -> Result<Self> {
        println!("Loading llama model from: {}", model_path.display());
        
        // Load the model using the static constructor pattern expected by trait
        let model = LlamaModel::load_from_file(
            model_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in model path"))?,
            LlamaParams::default()
        )?;
        
        Ok(Self::new(model))
    }

    fn prompt(&mut self, prompt: &str) -> Result<()> {
        // Stop any existing generation
        self.stop_generation();
        
        // Create a new session for this prompt
        let session = self.model.create_session(SessionParams::default())?;
        
        // Format the prompt according to TinyLlama's Zephyr format
        let formatted_prompt = format!(
            "<|system|>\nYou are a helpful assistant.</s>\n<|user|>\n{}</s>\n<|assistant|>\n",
            prompt
        );
        
        // Advance context with the formatted prompt
        let mut session = session;
        session.advance_context(&formatted_prompt)?;
        
        // Set up channels for token communication
        let (token_sender, token_receiver) = mpsc::channel();
        let (stop_sender, stop_receiver) = mpsc::channel();
        
        // Spawn worker thread to handle completion
        let worker_handle = thread::spawn(move || {
            println!("Worker thread: Starting completion...");
            // Start completing with standard sampler
            match session.start_completing_with(StandardSampler::default(), 1024) {
                Ok(completions) => {
                    println!("Worker thread: Successfully started completion");
                    let mut completion_iter = completions.into_strings();
                    
                    // Send tokens until we're told to stop or completion finishes
                    loop {
                        // Check if we should stop
                        if stop_receiver.try_recv().is_ok() {
                            println!("Worker thread: Stop signal received");
                            break;
                        }
                        
                        // Get next completion chunk
                        match completion_iter.next() {
                            Some(token) => {
                                println!("Worker thread: Generated token: '{}'", token);
                                if token_sender.send(Some(token)).is_err() {
                                    println!("Worker thread: Receiver dropped, stopping");
                                    break; // Receiver dropped
                                }
                            }
                            None => {
                                println!("Worker thread: Completion finished");
                                // Completion finished, send None to signal end
                                let _ = token_sender.send(None);
                                break;
                            }
                        }
                    }
                },
                Err(e) => {
                    println!("Worker thread: Error starting completion: {:?}", e);
                    // Error starting completion, send None to signal end
                    let _ = token_sender.send(None);
                }
            }
            println!("Worker thread: Exiting");
        });
        
        // Store the communication channels and worker handle
        self.session = Some(self.model.create_session(SessionParams::default())?); // Keep a session reference
        self.token_receiver = Some(token_receiver);
        self.worker_handle = Some(worker_handle);
        self.stop_sender = Some(stop_sender);
        
        Ok(())
    }

    fn next_token(&mut self) -> Result<Option<String>> {
        if let Some(receiver) = &self.token_receiver {
            match receiver.recv() {
                Ok(token) => Ok(token),
                Err(_) => {
                    // Channel closed, generation finished
                    self.stop_generation();
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    fn unload(&mut self) -> Result<()> {
        // Stop any ongoing generation
        self.stop_generation();
        
        // Clear session
        self.session = None;
        
        println!("Unloaded llama model");
        Ok(())
    }
} 