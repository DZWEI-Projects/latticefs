use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_env(temp: &TempDir) -> (PathBuf, PathBuf) {
    let lattice_home = temp.path().join(".latticefs");
    let xdg_home = temp.path().join("xdg");
    fs::create_dir_all(&lattice_home).unwrap();
    fs::create_dir_all(&xdg_home).unwrap();
    (lattice_home, xdg_home)
}

fn lfs_cmd(lattice_home: &PathBuf, xdg_home: &PathBuf) -> Command {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("lfs"));
    cmd.env("LATTICE_HOME", lattice_home);
    cmd.env("XDG_CONFIG_HOME", xdg_home);
    cmd.env("LFS_KEY_PASSWORD", "test-password");
    cmd
}

#[test]
fn cli_flow_basic() {
    let temp = TempDir::new().unwrap();
    let (lattice_home, xdg_home) = setup_env(&temp);

    // init
    lfs_cmd(&lattice_home, &xdg_home)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized repository"));

    // create a file
    let file_path = temp.path().join("hello.txt");
    fs::write(&file_path, b"hello latticefs\n").unwrap();

    // add file
    let output = lfs_cmd(&lattice_home, &xdg_home)
        .args(["add", file_path.to_str().unwrap(), "--tag", "project:phoenix"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let object_id = stdout
        .split_whitespace()
        .last()
        .expect("object id")
        .to_string();
    uuid::Uuid::parse_str(&object_id).expect("valid uuid");

    // tags list
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["tags", &object_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("project:phoenix"));

    // meta (auto tags + text)
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["meta", &object_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("auto:mimetype:text/plain"))
        .stdout(predicate::str::contains("hello latticefs"));

    // add an image to exercise auto:mimetype wildcards
    let image_path = temp.path().join("photo.jpg");
    fs::write(&image_path, b"fakejpg").unwrap();
    let output = lfs_cmd(&lattice_home, &xdg_home)
        .args(["add", image_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let image_id = stdout
        .split_whitespace()
        .last()
        .expect("image object id")
        .to_string();
    uuid::Uuid::parse_str(&image_id).expect("valid uuid");

    lfs_cmd(&lattice_home, &xdg_home)
        .args(["meta", &image_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("auto:mimetype:image/jpeg"));

    // view create (auto tag wildcard)
    lfs_cmd(&lattice_home, &xdg_home)
        .args([
            "view",
            "create",
            "ImageAuto",
            "--query",
            "tag:auto:mimetype:image/*",
        ])
        .assert()
        .success();

    lfs_cmd(&lattice_home, &xdg_home)
        .args(["stats", "view", "ImageAuto"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Objects: 1"));

    // view create
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["view", "create", "Images", "--query", "type:text/plain"])
        .assert()
        .success();

    // view list
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["view", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Images"));

    // view explain
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["view", "explain", &object_id, "--query", "tag:project:phoenix"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tag:project:phoenix"));

    // export
    let export_path = temp.path().join("out.txt");
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["export", &object_id, "--output", export_path.to_str().unwrap()])
        .assert()
        .success();
    let exported = fs::read(&export_path).unwrap();
    assert_eq!(exported, b"hello latticefs\n");

    // revise content (new version)
    fs::write(&file_path, b"hello latticefs v2\n").unwrap();
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["revise", &object_id, file_path.to_str().unwrap(), "-m", "update"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Revised"));

    // set state to review (v2)
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["state", "set", &format!("{}@v2", object_id), "review"])
        .assert()
        .success()
        .stdout(predicate::str::contains("draft -> review"));

    // revise content via stdin (new version, auto-advance v2 -> approved)
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["revise", &object_id, "--stdin", "-m", "stdin update"])
        .write_stdin("hello latticefs v3\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Revised"));

    // revise again (auto-advance v3 draft -> discarded)
    fs::write(&file_path, b"hello latticefs v4\n").unwrap();
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["revise", &object_id, file_path.to_str().unwrap(), "-m", "final update"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Revised"));

    // seal current version (v4) and ensure updates are blocked
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["state", "set", &format!("{}@v4", object_id), "sealed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sealed"));
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["revise", &object_id, file_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("sealed"));

    // versions
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["versions", &object_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("v4"))
        .stdout(predicate::str::contains("state=approved"))
        .stdout(predicate::str::contains("state=discarded"));

    // trust set/get
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["trust", "set", &object_id, "quarantined"])
        .assert()
        .success();
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["trust", "get", &object_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("quarantined"));

    // diff (explicit refs)
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["diff", &format!("{}@v1", object_id), &format!("{}@v2", object_id)])
        .assert()
        .success()
        .stdout(predicate::str::contains("---"));

    // diff shorthand (same object)
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["diff", &object_id, "v2", "v2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No differences"));

    // diff across objects
    let other_path = temp.path().join("other.txt");
    fs::write(&other_path, b"hello different\n").unwrap();
    let other_output = lfs_cmd(&lattice_home, &xdg_home)
        .args(["add", other_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(other_output.status.success());
    let other_stdout = String::from_utf8_lossy(&other_output.stdout);
    let other_id = other_stdout
        .split_whitespace()
        .last()
        .expect("object id")
        .to_string();

    lfs_cmd(&lattice_home, &xdg_home)
        .args([&format!("diff"), &format!("{}@v1", object_id), &format!("{}@v1", other_id)])
        .assert()
        .success()
        .stdout(predicate::str::contains("---"));
}
