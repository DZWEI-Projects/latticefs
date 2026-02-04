use crate::crypto::{Capability, Facts, Permission, PublicKey};
use crate::error::{LatticeError, Result};
use crate::events::Event;
use crate::ipc::{bind_listener, recv_message, send_message, socket_path, MessageType};
use crate::repo::LatticeRepo;
use crate::security::is_quarantined_executable;
use crate::KeyManager;
use prost::Message;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UnixStream;
use tokio::sync::watch;

use super::proto as pb;

pub async fn run_ipc_server(repo: LatticeRepo) -> Result<()> {
    let repo = Arc::new(repo);
    let socket_path = socket_path(&repo);
    let listener = bind_listener(&repo).await?;

    if repo.config.ipc.verbose {
        eprintln!("✓ IPC server started successfully");
        eprintln!("  {:<18} {}", "Socket:", socket_path.display());
        eprintln!("  {:<18} {}", "Protocol:", "Unix domain socket");
        eprintln!("  {:<18} {}", "Message types:", "ShareRequest, RevokeRequest, FetchRequest, StatusRequest, ShutdownRequest, SyncEvent");
        eprintln!(
            "  {:<18} {} MiB",
            "Max message size:",
            crate::ipc::MAX_MESSAGE_SIZE / (1024 * 1024)
        );
        eprintln!("  Listening for connections...");
    }

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    break;
                }
            }
            accept = listener.accept() => {
                let (stream, _) = accept?;
                let repo = Arc::clone(&repo);
                let shutdown_tx = shutdown_tx.clone();
                tokio::spawn(async move {
                    let _ = handle_connection(repo, stream, shutdown_tx).await;
                });
            }
        }
    }

    Ok(())
}

async fn handle_connection(
    repo: Arc<LatticeRepo>,
    mut stream: UnixStream,
    shutdown_tx: watch::Sender<bool>,
) -> Result<()> {
    loop {
        let (msg_type, payload) = match recv_message(&mut stream).await {
            Ok(msg) => msg,
            Err(err) => return Err(err),
        };

        match msg_type {
            MessageType::ShareRequest => {
                let request = pb::ShareRequest::decode(payload)?;
                let response = handle_share_request(&repo, request).await;
                send_message(&mut stream, MessageType::ShareResponse, &response).await?;
            }
            MessageType::RevokeRequest => {
                let request = pb::RevokeRequest::decode(payload)?;
                let response = handle_revoke_request(&repo, request).await;
                send_message(&mut stream, MessageType::RevokeResponse, &response).await?;
            }
            MessageType::FetchRequest => {
                let request = pb::FetchRequest::decode(payload)?;
                let response = handle_fetch_request(&repo, request).await;
                send_message(&mut stream, MessageType::FetchResponse, &response).await?;
            }
            MessageType::StatusRequest => {
                let request = pb::StatusRequest::decode(payload)?;
                let response = handle_status_request(&repo, request).await?;
                send_message(&mut stream, MessageType::StatusResponse, &response).await?;
            }
            MessageType::ShutdownRequest => {
                let _request = pb::ShutdownRequest::decode(payload)?;
                let response = pb::ShutdownResponse { success: true };
                send_message(&mut stream, MessageType::ShutdownResponse, &response).await?;
                let _ = shutdown_tx.send(true);
                break;
            }
            MessageType::SyncEvent => {
                let event = pb::SyncEvent::decode(payload)?;
                let ack = pb::SyncAck {
                    event_timestamp: event.timestamp,
                    success: true,
                };
                send_message(&mut stream, MessageType::SyncAck, &ack).await?;
            }
            _ => {
                let err = pb::Error {
                    code: 1,
                    message: format!("Unexpected message type: {:?}", msg_type),
                    details: HashMap::new(),
                };
                let response = pb::ShareResponse {
                    result: Some(pb::share_response::Result::Error(err)),
                };
                send_message(&mut stream, MessageType::ShareResponse, &response).await?;
            }
        }
    }

    Ok(())
}

async fn handle_share_request(repo: &LatticeRepo, request: pb::ShareRequest) -> pb::ShareResponse {
    let result = (|| {
        repo.enforce_rate_limit(1)?;
        let object_id = object_id_from_bytes(
            &request
                .object_id
                .ok_or_else(|| LatticeError::Serialization("Missing object_id".to_string()))?,
        )?;
        let object = repo.metadata.load_object(&object_id)?;

        let permission: Permission = request.capability.parse()?;
        repo.authorize_object_permission(&object, Permission::Share, true)?;
        repo.authorize_object_permission(&object, permission, false)?;

        let audience = PublicKey::from_did(&request.audience_did)?;
        let expires_at = request
            .expires_at
            .ok_or_else(|| LatticeError::Serialization("Missing expires_at".to_string()))?;
        let expires_in = duration_until(expires_at.micros);
        let facts = facts_from_map(request.facts);

        let identity = load_default_identity()?;
        let capability = Capability::create_with_facts(
            &identity, &audience, &object_id, permission, expires_in, facts,
        )?;
        repo.metadata.store_capability(&capability)?;
        repo.events.emit_sync(Event::share_issued(
            format!("latticefs:object:{}", object_id),
            capability.cid(),
            request.audience_did.clone(),
            capability.expires_at(),
        ));

        Ok(capability.token)
    })();

    match result {
        Ok(token) => pb::ShareResponse {
            result: Some(pb::share_response::Result::UcanToken(token)),
        },
        Err(err) => pb::ShareResponse {
            result: Some(pb::share_response::Result::Error(ipc_error(&err))),
        },
    }
}

async fn handle_revoke_request(
    repo: &LatticeRepo,
    request: pb::RevokeRequest,
) -> pb::RevokeResponse {
    let result = (|| {
        repo.enforce_rate_limit(1)?;
        let cap = repo.metadata.load_capability(&request.ucan_cid)?;
        let identity = load_default_identity()?;
        let revocation = cap.revoke(
            &identity,
            if request.reason.is_empty() {
                None
            } else {
                Some(request.reason.clone())
            },
            None,
            &repo.metadata,
        )?;
        repo.metadata.store_revocation(&revocation)?;
        Ok(())
    })();

    match result {
        Ok(()) => pb::RevokeResponse {
            result: Some(pb::revoke_response::Result::Success(true)),
        },
        Err(err) => pb::RevokeResponse {
            result: Some(pb::revoke_response::Result::Error(ipc_error(&err))),
        },
    }
}

async fn handle_fetch_request(repo: &LatticeRepo, request: pb::FetchRequest) -> pb::FetchResponse {
    let result: Result<pb::ObjectData> = async {
        repo.enforce_rate_limit(1)?;
        let object_id = object_id_from_bytes(
            &request
                .object_id
                .ok_or_else(|| LatticeError::Serialization("Missing object_id".to_string()))?,
        )?;

        let token = request.ucan_token.clone();
        let capability = Capability::parse(&token)?;
        capability.validate(&repo.metadata)?;
        if !capability.has_permission(&object_id, Permission::Read) {
            return Err(LatticeError::Unauthorized {
                permission: "read".to_string(),
                object: object_id.to_string(),
            });
        }

        let object = repo.metadata.load_object(&object_id)?;
        repo.authorize_object_permission(&object, Permission::Read, false)?;
        if is_quarantined_executable(&object.tags) {
            return Err(LatticeError::Unauthorized {
                permission: "read".to_string(),
                object: object_id.to_string(),
            });
        }

        let version_id = request
            .version_id
            .as_ref()
            .map(version_id_from_bytes)
            .transpose()?
            .unwrap_or(object.current_version);

        let version = repo.metadata.load_version(&version_id)?;
        if version.object_id != object_id {
            return Err(LatticeError::VersionNotFound {
                id: version_id.to_string(),
            });
        }

        let manifest = repo.metadata.load_manifest(&version.manifest_ref)?;
        let data = repo.chunks.retrieve_object(&manifest).await?;

        let tags = object
            .tags
            .iter()
            .map(|t| pb::Tag {
                key: t.key.clone(),
                value: t.value.clone(),
            })
            .collect();

        let object_data = pb::ObjectData {
            object_id: Some(pb::ObjectId {
                uuid: object_id.to_bytes(),
            }),
            version_id: Some(pb::VersionId {
                uuid: version_id.to_bytes(),
            }),
            content: data,
            metadata: HashMap::new(),
            tags,
        };

        Ok(object_data)
    }
    .await;

    match result {
        Ok(data) => pb::FetchResponse {
            result: Some(pb::fetch_response::Result::Data(data)),
        },
        Err(err) => pb::FetchResponse {
            result: Some(pb::fetch_response::Result::Error(ipc_error(&err))),
        },
    }
}

async fn handle_status_request(
    repo: &LatticeRepo,
    request: pb::StatusRequest,
) -> Result<pb::StatusResponse> {
    let stats = if request.include_stats {
        Some(build_stats(repo)?)
    } else {
        None
    };

    Ok(pb::StatusResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        running: true,
        stats,
    })
}

fn build_stats(repo: &LatticeRepo) -> Result<pb::Stats> {
    let total_objects = repo.metadata.iter_objects().count() as u64;
    let total_versions = repo.metadata.iter_all_versions().count() as u64;
    let (total_chunks, storage_bytes) = count_chunks(repo.root.join("chunks"));

    let logical_bytes: u64 = repo
        .metadata
        .iter_all_versions()
        .filter_map(|res| res.ok())
        .filter_map(|(_k, v)| bincode::deserialize::<crate::model::Version>(&v).ok())
        .map(|v| v.size_bytes)
        .sum();

    let dedup_ratio_x100 = if storage_bytes == 0 {
        100
    } else {
        ((logical_bytes as f64 / storage_bytes as f64) * 100.0).round() as u64
    };

    Ok(pb::Stats {
        total_objects,
        total_versions,
        total_chunks,
        storage_bytes,
        dedup_ratio_x100,
    })
}

fn count_chunks(root: std::path::PathBuf) -> (u64, u64) {
    let mut count = 0u64;
    let mut bytes = 0u64;
    if !root.exists() {
        return (0, 0);
    }

    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            count += 1;
            if let Ok(meta) = entry.metadata() {
                bytes = bytes.saturating_add(meta.len());
            }
        }
    }

    (count, bytes)
}

fn duration_until(target_micros: i64) -> Duration {
    let now = now_micros();
    if target_micros <= now {
        return Duration::from_secs(0);
    }
    let delta = (target_micros - now) as u64;
    Duration::from_micros(delta)
}

fn now_micros() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_micros() as i64
}

fn object_id_from_bytes(object_id: &pb::ObjectId) -> Result<crate::model::ObjectID> {
    let uuid = uuid::Uuid::from_slice(&object_id.uuid)
        .map_err(|e| LatticeError::Serialization(format!("Invalid object id bytes: {}", e)))?;
    Ok(crate::model::ObjectID::from_uuid(uuid))
}

fn version_id_from_bytes(version_id: &pb::VersionId) -> Result<crate::model::VersionID> {
    let uuid = uuid::Uuid::from_slice(&version_id.uuid)
        .map_err(|e| LatticeError::Serialization(format!("Invalid version id bytes: {}", e)))?;
    Ok(crate::model::VersionID::from_uuid(uuid))
}

fn facts_from_map(map: HashMap<String, String>) -> Option<Facts> {
    if map.is_empty() {
        return None;
    }

    let mut facts = Facts::default();
    for (key, value) in map {
        match key.as_str() {
            "lfs/version" => facts.version = Some(value),
            "lfs/device" => facts.device = Some(value),
            _ => {
                facts.custom.insert(key, serde_json::Value::String(value));
            }
        }
    }
    Some(facts)
}

fn load_default_identity() -> Result<crate::crypto::Identity> {
    let manager = KeyManager::auto();
    if manager.exists("default") {
        return manager.load("default", std::env::var("LFS_KEY_PASSWORD").ok().as_deref());
    }

    let identity = crate::crypto::Identity::generate("default");
    manager.store(&identity, std::env::var("LFS_KEY_PASSWORD").ok().as_deref())?;
    Ok(identity)
}

fn ipc_error(err: &LatticeError) -> pb::Error {
    let code = match err {
        LatticeError::ObjectNotFound { .. } => 2,
        LatticeError::VersionNotFound { .. } => 2,
        LatticeError::Unauthorized { .. } => 3,
        LatticeError::CapabilityExpired
        | LatticeError::CapabilityRevoked
        | LatticeError::CapabilityNotYetValid
        | LatticeError::InvalidSignature
        | LatticeError::InvalidProofChain(_)
        | LatticeError::InvalidAttenuation { .. } => 4,
        LatticeError::QuotaExceeded { .. } => 6,
        LatticeError::RateLimited { .. } => 6,
        _ => 99,
    };

    pb::Error {
        code,
        message: err.to_string(),
        details: HashMap::new(),
    }
}
