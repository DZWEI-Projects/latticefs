use anyhow::Result;
use clap::Args;
use latticefs_base::storage::hash_to_hex;
use latticefs_base::LatticeRepo;
use std::collections::HashSet;
use std::path::PathBuf;

use super::common::resolve_object_id;

#[derive(Args, Debug)]
pub struct InitArgs {}

#[derive(Args, Debug)]
pub struct StatusArgs {}

#[derive(Args, Debug)]
pub struct GcArgs {}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// Optional object reference
    pub reference: Option<String>,
    /// Deep verification (all versions)
    #[arg(long)]
    pub deep: bool,
}

pub async fn init(_args: InitArgs) -> Result<()> {
    let repo = LatticeRepo::init()?;
    println!("Initialized repository at {}", repo.root.display());
    Ok(())
}

pub async fn status(repo: LatticeRepo, _args: StatusArgs) -> Result<()> {
    let object_count = repo.metadata.iter_object_ids()?.len();
    let version_count = repo.metadata.iter_all_versions().count();

    let (chunk_count, chunk_bytes) = count_chunks(&repo.root.join("chunks"))?;

    println!("Objects: {}", object_count);
    println!("Versions: {}", version_count);
    println!("Chunks: {}", chunk_count);
    println!("Chunk bytes: {}", chunk_bytes);
    Ok(())
}

pub async fn gc(repo: LatticeRepo, _args: GcArgs) -> Result<()> {
    let referenced = collect_referenced_chunks(&repo)?;
    let mut removed = 0u64;

    for entry in walkdir::WalkDir::new(repo.root.join("chunks")).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy();
        if file_name.len() != 64 {
            continue;
        }
        if !referenced.contains(file_name.as_ref()) {
            std::fs::remove_file(entry.path())?;
            removed += 1;
        }
    }

    println!("GC removed {} chunks", removed);
    Ok(())
}

pub async fn verify(repo: LatticeRepo, args: VerifyArgs) -> Result<()> {
    if let Some(reference) = args.reference {
        let object_id = resolve_object_id(&repo, &reference)?;
        verify_object(&repo, &object_id, args.deep).await?;
        println!("Verified {}", object_id);
        return Ok(());
    }

    let mut checked = 0usize;
    for object in repo.metadata.iter_objects() {
        let object = object?;
        verify_object(&repo, &object.id, args.deep).await?;
        checked += 1;
    }

    println!("Verified {} objects", checked);
    Ok(())
}

async fn verify_object(repo: &LatticeRepo, object_id: &latticefs_base::ObjectID, deep: bool) -> Result<()> {
    let object = repo.metadata.load_object(object_id)?;
    if deep {
        for vid in object.versions {
            let version = repo.metadata.load_version(&vid)?;
            verify_version(repo, &version).await?;
        }
    } else {
        let version = repo.metadata.load_version(&object.current_version)?;
        verify_version(repo, &version).await?;
    }
    Ok(())
}

async fn verify_version(repo: &LatticeRepo, version: &latticefs_base::Version) -> Result<()> {
    let manifest = repo.metadata.load_manifest(&version.manifest_ref)?;
    for chunk in &manifest.chunks {
        let data = repo.chunks.read_chunk(&chunk.hash).await?;
        if data.len() != chunk.length as usize {
            return Err(anyhow::anyhow!("Chunk length mismatch"));
        }
    }
    let hashes: Vec<_> = manifest.chunks.iter().map(|c| c.hash).collect();
    let computed = latticefs_base::compute_merkle_root(&hashes);
    if computed != manifest.merkle_root {
        return Err(anyhow::anyhow!("Merkle root mismatch"));
    }
    Ok(())
}

fn collect_referenced_chunks(repo: &LatticeRepo) -> Result<HashSet<String>> {
    let mut set = HashSet::new();
    for item in repo.metadata.iter_all_versions() {
        let (k, _v) = item?;
        let version_id = latticefs_base::VersionID::from_bytes(&k)?;
        let version = repo.metadata.load_version(&version_id)?;
        let manifest = repo.metadata.load_manifest(&version.manifest_ref)?;
        for chunk in &manifest.chunks {
            set.insert(hash_to_hex(&chunk.hash));
        }
    }
    Ok(set)
}

fn count_chunks(path: &PathBuf) -> Result<(u64, u64)> {
    let mut count = 0u64;
    let mut bytes = 0u64;
    if !path.exists() {
        return Ok((0, 0));
    }
    for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            count += 1;
            bytes += entry.metadata()?.len();
        }
    }
    Ok((count, bytes))
}
