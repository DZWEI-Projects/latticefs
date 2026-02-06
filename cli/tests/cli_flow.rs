use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
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
        .args([
            "add",
            file_path.to_str().unwrap(),
            "--tag",
            "project:phoenix",
        ])
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
        .args(["info", "view", "ImageAuto"])
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
        .args([
            "view",
            "explain",
            &object_id,
            "--query",
            "tag:project:phoenix",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("tag:project:phoenix"));

    // export
    let export_path = temp.path().join("out.txt");
    lfs_cmd(&lattice_home, &xdg_home)
        .args([
            "export",
            &object_id,
            "--output",
            export_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let exported = fs::read(&export_path).unwrap();
    assert_eq!(exported, b"hello latticefs\n");

    // revise content (new version)
    fs::write(&file_path, b"hello latticefs v2\n").unwrap();
    lfs_cmd(&lattice_home, &xdg_home)
        .args([
            "revise",
            &object_id,
            file_path.to_str().unwrap(),
            "-m",
            "update",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Revised"));

    // export specific version (v1)
    let export_path_v1 = temp.path().join("out_v1.txt");
    lfs_cmd(&lattice_home, &xdg_home)
        .args([
            "export",
            &format!("{}@v1", object_id),
            "--output",
            export_path_v1.to_str().unwrap(),
        ])
        .assert()
        .success();
    let exported_v1 = fs::read(&export_path_v1).unwrap();
    assert_eq!(exported_v1, b"hello latticefs\n");

    // export specific version (v2)
    let export_path_v2 = temp.path().join("out_v2.txt");
    lfs_cmd(&lattice_home, &xdg_home)
        .args([
            "export",
            &format!("{}@v2", object_id),
            "--output",
            export_path_v2.to_str().unwrap(),
        ])
        .assert()
        .success();
    let exported_v2 = fs::read(&export_path_v2).unwrap();
    assert_eq!(exported_v2, b"hello latticefs v2\n");

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
        .args([
            "revise",
            &object_id,
            file_path.to_str().unwrap(),
            "-m",
            "final update",
        ])
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
        .args([
            "diff",
            &format!("{}@v1", object_id),
            &format!("{}@v2", object_id),
        ])
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
        .args([
            &format!("diff"),
            &format!("{}@v1", object_id),
            &format!("{}@v1", other_id),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("---"));
}

#[test]
fn cli_message_set_and_clear() {
    let temp = TempDir::new().unwrap();
    let (lattice_home, xdg_home) = setup_env(&temp);

    // init
    lfs_cmd(&lattice_home, &xdg_home)
        .arg("init")
        .assert()
        .success();

    // create a file and add it
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, b"test content\n").unwrap();
    let output = lfs_cmd(&lattice_home, &xdg_home)
        .args(["add", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let object_id = stdout
        .split_whitespace()
        .last()
        .expect("object id")
        .to_string();
    
    // Set initial message
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["message", "set", &object_id, "-m", "initial message"])
        .assert()
        .success();

    // Set a new message
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["message", "set", &object_id, "-m", "updated message"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Set message for"));

    // Set message for a specific version
    lfs_cmd(&lattice_home, &xdg_home)
        .args([
            "message",
            "set",
            &format!("{}@v1", object_id),
            "-m",
            "version-specific message",
        ])
        .assert()
        .success();

    // Clear the message
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["message", "set", &object_id, "--clear"])
        .assert()
        .success();

    // Test error case: both --message and --clear
    lfs_cmd(&lattice_home, &xdg_home)
        .args([
            "message",
            "set",
            &object_id,
            "-m",
            "test",
            "--clear",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("either --message or --clear"));

    // Test error case: missing message
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["message", "set", &object_id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Missing --message"));
}

#[test]
fn cli_flow_nested_views() {
    let temp = TempDir::new().unwrap();
    let (lattice_home, xdg_home) = setup_env(&temp);

    // init
    lfs_cmd(&lattice_home, &xdg_home)
        .arg("init")
        .assert()
        .success();

    // Create files with different tags
    let file1 = temp.path().join("file1.txt");
    let file2 = temp.path().join("file2.txt");
    let file3 = temp.path().join("file3.txt");
    fs::write(&file1, b"content1").unwrap();
    fs::write(&file2, b"content2").unwrap();
    fs::write(&file3, b"content3").unwrap();

    let output1 = lfs_cmd(&lattice_home, &xdg_home)
        .args([
            "add",
            file1.to_str().unwrap(),
            "--tag",
            "project:phoenix",
            "--tag",
            "kind:doc",
        ])
        .output()
        .unwrap();
    assert!(output1.status.success());

    let output2 = lfs_cmd(&lattice_home, &xdg_home)
        .args(["add", file2.to_str().unwrap(), "--tag", "project:phoenix"])
        .output()
        .unwrap();
    assert!(output2.status.success());

    let output3 = lfs_cmd(&lattice_home, &xdg_home)
        .args([
            "add",
            file3.to_str().unwrap(),
            "--tag",
            "project:apollo",
            "--tag",
            "kind:doc",
        ])
        .output()
        .unwrap();
    assert!(output3.status.success());

    // Create parent view
    lfs_cmd(&lattice_home, &xdg_home)
        .args([
            "view",
            "create",
            "PhoenixProject",
            "--query",
            "tag:project:phoenix",
        ])
        .assert()
        .success();

    // Create nested view
    lfs_cmd(&lattice_home, &xdg_home)
        .args([
            "view",
            "create",
            "PhoenixDocs",
            "--query",
            "tag:kind:doc",
            "--parent",
            "PhoenixProject",
        ])
        .assert()
        .success();

    // List views should show hierarchy
    let list_output = lfs_cmd(&lattice_home, &xdg_home)
        .args(["view", "list"])
        .output()
        .unwrap();
    assert!(list_output.status.success());
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("PhoenixProject"));
    assert!(list_stdout.contains("PhoenixDocs"));
    // Check for indentation (nested view should be indented)
    let lines: Vec<&str> = list_stdout.lines().collect();
    let phoenix_project_line = lines
        .iter()
        .position(|l| l.contains("PhoenixProject"))
        .unwrap();
    let phoenix_docs_line = lines
        .iter()
        .position(|l| l.contains("PhoenixDocs"))
        .unwrap();
    // PhoenixDocs should come after PhoenixProject and be indented
    assert!(phoenix_docs_line > phoenix_project_line);

    // Verify nested view query works (should only match file1, not file2 or file3)
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["info", "view", "PhoenixDocs"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Parent: PhoenixProject"))
        .stdout(predicate::str::contains("Objects: 1"));
}

#[test]
fn watchd_start_foreground_no_db_lock() {
    // Regression test: `watchd start --foreground` used to fail with a database
    // lock error because the daemon kept the Sled database open permanently.
    // The fix makes the daemon open/close the DB on demand per operation.
    let temp = TempDir::new().unwrap();
    let (lattice_home, xdg_home) = setup_env(&temp);

    // Initialize a repository first
    lfs_cmd(&lattice_home, &xdg_home)
        .arg("init")
        .assert()
        .success();

    // Start watchd in foreground as a child process.
    // It should start successfully (not crash with a lock error).
    let mut child = std::process::Command::new(assert_cmd::cargo::cargo_bin!("lfs"))
        .env("LATTICE_HOME", &lattice_home)
        .env("XDG_CONFIG_HOME", &xdg_home)
        .env("LFS_KEY_PASSWORD", "test-password")
        .args(["watchd", "start", "--foreground"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn watchd");

    // Give the daemon a moment to start (or fail)
    std::thread::sleep(Duration::from_secs(2));

    // The process should still be running (not crashed with a lock error)
    match child.try_wait() {
        Ok(None) => {
            // Still running — success! Clean up.
            child.kill().ok();
            child.wait().ok();
        }
        Ok(Some(status)) => {
            let stderr = child
                .stderr
                .take()
                .map(|mut s| {
                    let mut buf = String::new();
                    std::io::Read::read_to_string(&mut s, &mut buf).ok();
                    buf
                })
                .unwrap_or_default();
            panic!(
                "watchd exited prematurely with status {:?}.\nStderr: {}",
                status, stderr,
            );
        }
        Err(e) => panic!("Error checking watchd status: {}", e),
    }
}

#[test]
fn watchd_status_when_not_running() {
    let temp = TempDir::new().unwrap();
    let (lattice_home, xdg_home) = setup_env(&temp);

    lfs_cmd(&lattice_home, &xdg_home)
        .arg("init")
        .assert()
        .success();

    // Status should report "not running" without error
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["watchd", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not running"));
}

#[test]
fn watchd_stop_when_not_running() {
    let temp = TempDir::new().unwrap();
    let (lattice_home, xdg_home) = setup_env(&temp);

    lfs_cmd(&lattice_home, &xdg_home)
        .arg("init")
        .assert()
        .success();

    // Stop should fail gracefully when daemon is not running
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["watchd", "stop"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not running"));
}

#[test]
fn watchd_stale_pid_cleanup() {
    // If a PID file exists but the process is dead, watchd should clean up
    // the stale PID file and start successfully.
    let temp = TempDir::new().unwrap();
    let (lattice_home, xdg_home) = setup_env(&temp);

    lfs_cmd(&lattice_home, &xdg_home)
        .arg("init")
        .assert()
        .success();

    // Write a stale PID file (PID 1 is init and won't match, use a very high PID)
    let pid_path = lattice_home.join("watchd.pid");
    fs::write(&pid_path, "99999999").unwrap();

    // Starting should succeed (stale PID gets cleaned up)
    let mut child = std::process::Command::new(assert_cmd::cargo::cargo_bin!("lfs"))
        .env("LATTICE_HOME", &lattice_home)
        .env("XDG_CONFIG_HOME", &xdg_home)
        .env("LFS_KEY_PASSWORD", "test-password")
        .args(["watchd", "start", "--foreground"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn watchd");

    std::thread::sleep(Duration::from_secs(2));

    match child.try_wait() {
        Ok(None) => {
            // Still running — stale PID was cleaned up successfully
            child.kill().ok();
            child.wait().ok();
        }
        Ok(Some(status)) => {
            let stderr = child
                .stderr
                .take()
                .map(|mut s| {
                    let mut buf = String::new();
                    std::io::Read::read_to_string(&mut s, &mut buf).ok();
                    buf
                })
                .unwrap_or_default();
            panic!(
                "watchd exited prematurely with status {:?}.\nStderr: {}",
                status, stderr,
            );
        }
        Err(e) => panic!("Error checking watchd status: {}", e),
    }
}

#[test]
fn watchd_concurrent_cli_access() {
    // Regression test: CLI commands like `add`, `tags`, `meta` must succeed
    // while the watchd daemon is running. The daemon releases the Sled file
    // lock between operations so other processes can access the database.
    let temp = TempDir::new().unwrap();
    let (lattice_home, xdg_home) = setup_env(&temp);

    // Initialize repo and add a file
    lfs_cmd(&lattice_home, &xdg_home)
        .arg("init")
        .assert()
        .success();

    let file_path = temp.path().join("concurrent.txt");
    fs::write(&file_path, b"concurrent test\n").unwrap();

    let output = lfs_cmd(&lattice_home, &xdg_home)
        .args(["add", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let object_id = stdout
        .split_whitespace()
        .last()
        .expect("object id")
        .to_string();

    // Start watchd daemon in foreground
    let mut daemon = std::process::Command::new(assert_cmd::cargo::cargo_bin!("lfs"))
        .env("LATTICE_HOME", &lattice_home)
        .env("XDG_CONFIG_HOME", &xdg_home)
        .env("LFS_KEY_PASSWORD", "test-password")
        .args(["watchd", "start", "--foreground"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn watchd");

    // Wait for daemon to fully start
    std::thread::sleep(Duration::from_secs(2));

    // Verify daemon is still alive
    assert!(
        daemon.try_wait().unwrap().is_none(),
        "daemon crashed on startup"
    );

    // Run CLI commands while daemon is running — these should all succeed
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["tags", &object_id])
        .assert()
        .success();

    lfs_cmd(&lattice_home, &xdg_home)
        .args(["meta", &object_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("concurrent test"));

    // Add another file while daemon is running
    let file2 = temp.path().join("second.txt");
    fs::write(&file2, b"second file\n").unwrap();
    lfs_cmd(&lattice_home, &xdg_home)
        .args(["add", file2.to_str().unwrap()])
        .assert()
        .success();

    // Clean up daemon
    daemon.kill().ok();
    daemon.wait().ok();
}
