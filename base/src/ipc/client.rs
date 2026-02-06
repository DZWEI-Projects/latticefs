use crate::error::{LatticeError, Result};
use crate::ipc::{recv_message, send_message, MessageType};
use crate::model::ActorID;
use crate::storage::Hash;
use prost::Message;
use std::path::Path;
use tokio::net::UnixStream;

use super::proto as pb;

pub async fn send_watch_register(
    socket_path: &Path,
    temp_path: &Path,
    object_id: &str,
    actor_id: ActorID,
    content_hash: Hash,
    display_name: &str,
) -> Result<bool> {
    let mut stream = connect(socket_path).await?;

    let request = pb::WatchRegisterRequest {
        temp_path: temp_path.display().to_string(),
        object_id: object_id.to_string(),
        actor_id: actor_id.to_vec(),
        content_hash: content_hash.to_vec(),
        display_name: display_name.to_string(),
    };

    send_message(&mut stream, MessageType::WatchRegisterRequest, &request).await?;
    let (msg_type, payload) = recv_message(&mut stream).await?;

    if msg_type != MessageType::WatchRegisterResponse {
        return Err(LatticeError::WatcherError(format!(
            "Unexpected response type: {:?}",
            msg_type
        )));
    }

    let response = pb::WatchRegisterResponse::decode(payload)?;
    Ok(response.success)
}

pub async fn send_watch_unregister(socket_path: &Path, temp_path: &Path) -> Result<bool> {
    let mut stream = connect(socket_path).await?;

    let request = pb::WatchUnregisterRequest {
        temp_path: temp_path.display().to_string(),
    };

    send_message(&mut stream, MessageType::WatchUnregisterRequest, &request).await?;
    let (msg_type, payload) = recv_message(&mut stream).await?;

    if msg_type != MessageType::WatchUnregisterResponse {
        return Err(LatticeError::WatcherError(format!(
            "Unexpected response type: {:?}",
            msg_type
        )));
    }

    let response = pb::WatchUnregisterResponse::decode(payload)?;
    Ok(response.success)
}

pub async fn send_watch_list(socket_path: &Path) -> Result<Vec<pb::WatchedFileInfo>> {
    let mut stream = connect(socket_path).await?;

    let request = pb::WatchListRequest {};
    send_message(&mut stream, MessageType::WatchListRequest, &request).await?;
    let (msg_type, payload) = recv_message(&mut stream).await?;

    if msg_type != MessageType::WatchListResponse {
        return Err(LatticeError::WatcherError(format!(
            "Unexpected response type: {:?}",
            msg_type
        )));
    }

    let response = pb::WatchListResponse::decode(payload)?;
    Ok(response.files)
}

pub async fn send_watch_status(socket_path: &Path) -> Result<pb::WatchStatusResponse> {
    let mut stream = connect(socket_path).await?;

    let request = pb::WatchStatusRequest {};
    send_message(&mut stream, MessageType::WatchStatusRequest, &request).await?;
    let (msg_type, payload) = recv_message(&mut stream).await?;

    if msg_type != MessageType::WatchStatusResponse {
        return Err(LatticeError::WatcherError(format!(
            "Unexpected response type: {:?}",
            msg_type
        )));
    }

    let response = pb::WatchStatusResponse::decode(payload)?;
    Ok(response)
}

pub fn is_daemon_running(socket_path: &Path) -> bool {
    socket_path.exists()
        && std::os::unix::net::UnixStream::connect(socket_path).is_ok()
}

async fn connect(socket_path: &Path) -> Result<UnixStream> {
    if !socket_path.exists() {
        return Err(LatticeError::WatcherNotRunning);
    }

    UnixStream::connect(socket_path)
        .await
        .map_err(|_| LatticeError::WatcherNotRunning)
}
