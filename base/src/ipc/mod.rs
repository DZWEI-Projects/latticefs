use crate::error::{LatticeError, Result};
use crate::repo::LatticeRepo;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

pub mod proto {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ipc/proto.rs"));
}

pub mod client;
pub mod server;

/// Maximum IPC message size (100 MiB).
pub const MAX_MESSAGE_SIZE: u32 = 100 * 1024 * 1024;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Unknown = 0,
    ShareRequest = 101,
    ShareResponse = 102,
    RevokeRequest = 103,
    RevokeResponse = 104,
    FetchRequest = 105,
    FetchResponse = 106,
    SyncEvent = 201,
    SyncAck = 202,
    StatusRequest = 301,
    StatusResponse = 302,
    ShutdownRequest = 303,
    ShutdownResponse = 304,
    WatchRegisterRequest = 401,
    WatchRegisterResponse = 402,
    WatchUnregisterRequest = 403,
    WatchUnregisterResponse = 404,
    WatchListRequest = 405,
    WatchListResponse = 406,
    WatchStatusRequest = 407,
    WatchStatusResponse = 408,
    Error = 999,
}

impl MessageType {
    pub fn from_u16(value: u16) -> Result<Self> {
        let msg = match value {
            0 => MessageType::Unknown,
            101 => MessageType::ShareRequest,
            102 => MessageType::ShareResponse,
            103 => MessageType::RevokeRequest,
            104 => MessageType::RevokeResponse,
            105 => MessageType::FetchRequest,
            106 => MessageType::FetchResponse,
            201 => MessageType::SyncEvent,
            202 => MessageType::SyncAck,
            301 => MessageType::StatusRequest,
            302 => MessageType::StatusResponse,
            303 => MessageType::ShutdownRequest,
            304 => MessageType::ShutdownResponse,
            401 => MessageType::WatchRegisterRequest,
            402 => MessageType::WatchRegisterResponse,
            403 => MessageType::WatchUnregisterRequest,
            404 => MessageType::WatchUnregisterResponse,
            405 => MessageType::WatchListRequest,
            406 => MessageType::WatchListResponse,
            407 => MessageType::WatchStatusRequest,
            408 => MessageType::WatchStatusResponse,
            999 => MessageType::Error,
            _ => return Err(LatticeError::InvalidPredicate(format!("Unknown message type: {}", value))),
        };
        Ok(msg)
    }
}

pub async fn send_message<T: prost::Message>(
    stream: &mut UnixStream,
    msg_type: MessageType,
    message: &T,
) -> Result<()> {
    let payload = message.encode_to_vec();
    let length = (2 + payload.len()) as u32;
    stream.write_all(&length.to_be_bytes()).await?;
    stream.write_all(&(msg_type as u16).to_be_bytes()).await?;
    stream.write_all(&payload).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn recv_message(stream: &mut UnixStream) -> Result<(MessageType, BytesMut)> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let length = u32::from_be_bytes(len_buf);
    if length > MAX_MESSAGE_SIZE {
        return Err(LatticeError::Serialization(format!(
            "Message too large: {}",
            length
        )));
    }

    let mut type_buf = [0u8; 2];
    stream.read_exact(&mut type_buf).await?;
    let msg_type = MessageType::from_u16(u16::from_be_bytes(type_buf))?;
    let payload_len = length.saturating_sub(2) as usize;
    let mut buf = BytesMut::with_capacity(payload_len);
    buf.resize(payload_len, 0);
    stream.read_exact(&mut buf).await?;
    Ok((msg_type, buf))
}

/// Start the IPC server.
pub async fn start_ipc_server(repo: LatticeRepo) -> Result<()> {
    server::run_ipc_server(repo).await
}

/// Get IPC socket path for the current repo config.
pub fn socket_path(repo: &LatticeRepo) -> std::path::PathBuf {
    repo.config.socket_path()
}

/// Bind and start a UnixListener on the IPC socket.
pub async fn bind_listener(repo: &LatticeRepo) -> Result<UnixListener> {
    let socket_path = socket_path(repo);

    if socket_path.exists() {
        tokio::fs::remove_file(&socket_path).await?;
    }

    let listener = UnixListener::bind(&socket_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&socket_path).await?.permissions();
        perms.set_mode(0o600);
        tokio::fs::set_permissions(&socket_path, perms).await?;
    }

    Ok(listener)
}
