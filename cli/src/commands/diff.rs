use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::LatticeRepo;
use similar::TextDiff;

use super::common::parse_ref_with_version;

#[derive(Args, Debug)]
pub struct DiffArgs {
    /// First reference (ref@version)
    pub left: String,
    /// Second reference (ref@version)
    pub right: String,
}

pub async fn run(repo: LatticeRepo, args: DiffArgs) -> Result<()> {
    let left = read_ref_with_version(&repo, &args.left).await?;
    let right = read_ref_with_version(&repo, &args.right).await?;

    if left == right {
        println!("No differences");
        return Ok(());
    }

    let left_text = String::from_utf8(left.clone());
    let right_text = String::from_utf8(right.clone());

    match (left_text, right_text) {
        (Ok(l), Ok(r)) => {
            let diff = TextDiff::from_lines(&l, &r);
            println!("{}", diff.unified_diff().header("left", "right"));
        }
        _ => {
            let min_len = std::cmp::min(left.len(), right.len());
            let mut first_diff = None;
            for i in 0..min_len {
                if left[i] != right[i] {
                    first_diff = Some(i);
                    break;
                }
            }
            println!("Binary diff: left={} bytes right={} bytes", left.len(), right.len());
            if let Some(pos) = first_diff {
                println!("First differing byte at offset {}", pos);
            }
        }
    }

    Ok(())
}

async fn read_ref_with_version(repo: &LatticeRepo, reference: &str) -> Result<Vec<u8>> {
    let (object_id, version_id) = parse_ref_with_version(repo, reference)?;
    let Some(version_id) = version_id else {
        return Err(anyhow::anyhow!("diff requires explicit versions (ref@version)"));
    };
    let _object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    let version = repo.metadata.load_version(&version_id)?;
    let manifest = repo.metadata.load_manifest(&version.manifest_ref)?;
    let data = repo.chunks.retrieve_object(&manifest).await?;
    Ok(data)
}
