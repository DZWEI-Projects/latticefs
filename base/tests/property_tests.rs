use base::{
    chunk_data, compute_hash, MetadataStore, Object, ObjectType, Permission, Policy, PolicyContext,
    PolicyEngine, Requirement, Version, VersionDAG, VersionID,
};
use proptest::prelude::*;
use std::collections::{BTreeSet, HashSet};
use std::path::Path;
use tempfile::tempdir;

#[derive(Debug, Clone)]
struct PolicyData {
    allow: Vec<Permission>,
    deny: Vec<Permission>,
    require: Vec<Requirement>,
    external_share: bool,
}

fn permission_strategy() -> impl Strategy<Value = Permission> {
    prop_oneof![
        Just(Permission::Read),
        Just(Permission::Comment),
        Just(Permission::Write),
        Just(Permission::Share),
        Just(Permission::Admin),
    ]
}

fn requirement_strategy() -> impl Strategy<Value = Requirement> {
    let approval = proptest::collection::vec("[a-z]{1,8}", 0..3)
        .prop_map(Requirement::ApprovalFrom);
    let min_trust = (0u8..=100).prop_map(Requirement::MinTrust);
    let required_tag = "[a-z]{1,8}".prop_map(Requirement::RequireTag);
    prop_oneof![approval, min_trust, required_tag]
}

fn policy_data_strategy() -> impl Strategy<Value = PolicyData> {
    (
        proptest::collection::vec(permission_strategy(), 0..5),
        proptest::collection::vec(permission_strategy(), 0..5),
        proptest::collection::vec(requirement_strategy(), 0..3),
        any::<bool>(),
    )
        .prop_map(|(allow, deny, require, external_share)| PolicyData {
            allow,
            deny,
            require,
            external_share,
        })
}

fn dedup_permissions(perms: Vec<Permission>) -> Vec<Permission> {
    let mut set = BTreeSet::new();
    for perm in perms {
        set.insert(perm);
    }
    set.into_iter().collect()
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0;
    let mut stack = vec![path.to_path_buf()];
    while let Some(current) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if let Ok(metadata) = entry.metadata() {
                    total += metadata.len();
                }
            }
        }
    }
    total
}

fn build_object() -> Object {
    let actor = [0u8; 32];
    let version_id = VersionID::new();
    Object::new(ObjectType::Blob, version_id, actor)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    #[test]
    fn chunking_is_deterministic(data in proptest::collection::vec(any::<u8>(), 0..65536)) {
        let chunks1 = chunk_data(&data);
        let chunks2 = chunk_data(&data);
        prop_assert_eq!(chunks1, chunks2);
    }

    #[test]
    fn duplicate_chunks_share_storage(data in proptest::collection::vec(any::<u8>(), 0..65536)) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let tmp = tempdir().expect("tempdir");
        let store = base::ChunkStore::new(tmp.path().to_path_buf());

        runtime.block_on(async {
            store.store_object(&data).await.expect("store first");
        });
        let size_before = dir_size(&tmp.path().join("chunks"));

        runtime.block_on(async {
            store.store_object(&data).await.expect("store second");
        });
        let size_after = dir_size(&tmp.path().join("chunks"));

        prop_assert_eq!(size_before, size_after);
    }

    #[test]
    fn policies_never_grant_access(policies in proptest::collection::vec(policy_data_strategy(), 1..20)) {
        let engine = PolicyEngine::new();
        let object = build_object();
        let context = PolicyContext::for_object(&object);

        let mut constructed = Vec::new();
        for (idx, data) in policies.iter().enumerate() {
            let mut policy = Policy::new(format!("policy-{}", idx));
            policy.allow = dedup_permissions(data.allow.clone());
            policy.deny = dedup_permissions(data.deny.clone());
            policy.require = data.require.clone();
            policy.external_share = data.external_share;
            constructed.push(policy);
        }

        let mut prev_allowed: Option<BTreeSet<Permission>> = None;
        for count in 1..=constructed.len() {
            let decision = engine.evaluate(&constructed[..count], &context);
            if let Some(prev) = prev_allowed.as_ref() {
                prop_assert!(decision.allowed.is_subset(prev));
            }
            prev_allowed = Some(decision.allowed.clone());
        }
    }

    #[test]
    fn version_dag_has_no_cycles(seed in proptest::collection::vec(any::<u8>(), 1..50)) {
        let temp = tempdir().expect("tempdir");
        let store = MetadataStore::open(temp.path()).expect("metadata");
        let object = build_object();
        store.store_object(&object).expect("store object");

        let mut versions: Vec<Version> = Vec::new();
        for (idx, value) in seed.iter().enumerate() {
            let parent = if idx == 0 {
                None
            } else {
                let offset = (*value as usize) % (idx + 1);
                if offset == 0 {
                    None
                } else {
                    Some(versions[idx - offset].id)
                }
            };

            let hash = compute_hash(&[*value]);
            let version = Version::new(
                object.id,
                parent,
                hash,
                hash,
                [0u8; 32],
                0,
                0,
                None,
            );
            store.store_version(&version).expect("store version");
            versions.push(version);
        }

        let dag = VersionDAG::new(&store);
        let mut seen = HashSet::new();
        for version in &versions {
            let ancestors = dag.ancestors(version.id).expect("ancestors");
            prop_assert!(!ancestors.contains(&version.id));
            for ancestor in ancestors {
                prop_assert!(seen.insert((version.id, ancestor)));
            }
        }
    }
}
