use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use threadrunner_core::error::{Error, Result};

/// Read a length-prefixed frame from the stream
pub async fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>> {
    // Read 4-byte length prefix
    let mut length_bytes = [0u8; 4];
    stream.read_exact(&mut length_bytes).await.map_err(|e| Error::Io(e))?;
    
    // Convert from little-endian u32
    let length = u32::from_le_bytes(length_bytes) as usize;
    
    // Read the actual data
    let mut data = vec![0u8; length];
    stream.read_exact(&mut data).await.map_err(|e| Error::Io(e))?;
    
    Ok(data)
}

/// Write a length-prefixed frame to the stream
pub async fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<()> {
    // Write 4-byte length prefix in little-endian
    let length = bytes.len() as u32;
    stream.write_all(&length.to_le_bytes()).await.map_err(|e| Error::Io(e))?;
    
    // Write the actual data
    stream.write_all(bytes).await.map_err(|e| Error::Io(e))?;
    
    Ok(())
} 