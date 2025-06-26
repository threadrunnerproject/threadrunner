use threadrunner_core::error::Error;
use std::io::{Error as IoError, ErrorKind};
use anyhow::anyhow;

#[test]
fn test_io_error_conversion() {
    // Test that std::io::ErrorKind::BrokenPipe maps via Error::from to Error::Io
    let io_error = IoError::new(ErrorKind::BrokenPipe, "broken pipe");
    let converted_error = Error::from(io_error);
    
    match converted_error {
        Error::Io(inner_error) => {
            assert_eq!(inner_error.kind(), ErrorKind::BrokenPipe);
        }
        _ => panic!("Expected Error::Io variant, got {:?}", converted_error),
    }
}

#[test]
fn test_anyhow_error_conversion() {
    // Test that custom anyhow!("oops") bubbles to Error::ModelLoad
    // Simulate via map_err(Error::from)
    let result: Result<(), anyhow::Error> = Err(anyhow!("oops"));
    let converted_result = result.map_err(Error::from);
    
    match converted_result {
        Err(Error::ModelLoad(inner_error)) => {
            assert_eq!(inner_error.to_string(), "oops");
        }
        _ => panic!("Expected Error::ModelLoad variant, got {:?}", converted_result),
    }
} 