use anyhow::{Context, Result};
use clap::Args;
use latticefs_base::LatticeRepo;
use latticefs_base::{is_quarantined_executable, Permission};
use similar::TextDiff;

use super::common::parse_ref_with_version;

#[derive(Args, Debug)]
pub struct DiffArgs {
    /// References to diff. Use either:
    ///   lfs diff <ref@v1> <ref@v2>
    /// or
    ///   lfs diff <ref> <v1> <v2>
    #[arg(required = true, num_args = 2..=3)]
    pub refs: Vec<String>,
}

pub async fn run(repo: LatticeRepo, args: DiffArgs) -> Result<()> {
    let (left_ref, right_ref) = normalize_refs(&args.refs)?;
    let left = read_ref_with_version(&repo, &left_ref).await?;
    let right = read_ref_with_version(&repo, &right_ref).await?;

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

fn normalize_refs(refs: &[String]) -> Result<(String, String)> {
    if refs.len() == 2 {
        return Ok((refs[0].clone(), refs[1].clone()));
    }

    if refs.len() == 3 {
        let object_ref = refs[0].trim();
        let v1 = refs[1].trim();
        let v2 = refs[2].trim();

        if object_ref.is_empty() || v1.is_empty() || v2.is_empty() {
            return Err(anyhow::anyhow!("diff requires a reference and two versions"));
        }
        if v1.contains('@') || v2.contains('@') {
            return Err(anyhow::anyhow!(
                "diff with three arguments expects versions only (no '@')"
            ));
        }

        let left = format!("{}@{}", object_ref, v1);
        let right = format!("{}@{}", object_ref, v2);
        return Ok((left, right));
    }

    Err(anyhow::anyhow!("diff requires two or three arguments"))
}

async fn read_ref_with_version(repo: &LatticeRepo, reference: &str) -> Result<Vec<u8>> {
    let (object_id, version_id) = parse_ref_with_version(repo, reference)?;
    let Some(version_id) = version_id else {
        return Err(anyhow::anyhow!("diff requires explicit versions (ref@version)"));
    };
    let object = repo
        .metadata
        .load_object(&object_id)
        .with_context(|| format!("Object not found: {}", object_id))?;
    repo.authorize_object_permission(&object, Permission::Read, false)?;
    if is_quarantined_executable(&object.tags) {
        return Err(anyhow::anyhow!("Object is quarantined and executable"));
    }
    let version = repo.metadata.load_version(&version_id)?;
    let manifest = repo.metadata.load_manifest(&version.manifest_ref)?;
    let data = repo.chunks.retrieve_object(&manifest).await?;
    Ok(data)
}
