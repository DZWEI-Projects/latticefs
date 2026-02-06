use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

const LOCAL_REPO_CONFIG_FILE: &str = ".latticefs.toml";

fn lfs_cmd(lattice_home: &PathBuf, xdg_home: &PathBuf) -> Command {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("lfs"));
    cmd.env("LATTICE_HOME", lattice_home);
    cmd.env("XDG_CONFIG_HOME", xdg_home);
    cmd.env("LFS_KEY_PASSWORD", "test-password");
    cmd
}

#[test]
fn auto_loads_repo_from_current_directory_marker() {
    let temp = TempDir::new().unwrap();
    let workspace = temp.path().join("workspace");
    let lattice_home = temp.path().join("default-home");
    let xdg_home = temp.path().join("xdg");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&lattice_home).unwrap();
    fs::create_dir_all(&xdg_home).unwrap();

    fs::write(
        workspace.join(LOCAL_REPO_CONFIG_FILE),
        "[repo]\nauto_load = true\n",
    )
    .unwrap();

    lfs_cmd(&lattice_home, &xdg_home)
        .current_dir(&workspace)
        .arg("status")
        .assert()
        .success();

    assert!(workspace.join("meta").is_dir());
    assert!(workspace.join("chunks").is_dir());
    assert!(workspace.join("logs").is_dir());
    assert!(!lattice_home.join("meta").exists());
}

#[test]
fn explicit_repo_flag_overrides_auto_load_marker() {
    let temp = TempDir::new().unwrap();
    let workspace = temp.path().join("workspace");
    let explicit_repo = temp.path().join("explicit-repo");
    let lattice_home = temp.path().join("default-home");
    let xdg_home = temp.path().join("xdg");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&lattice_home).unwrap();
    fs::create_dir_all(&xdg_home).unwrap();

    fs::write(
        workspace.join(LOCAL_REPO_CONFIG_FILE),
        "[repo]\nauto_load = true\n",
    )
    .unwrap();

    lfs_cmd(&lattice_home, &xdg_home)
        .current_dir(&workspace)
        .args(["--repo", explicit_repo.to_str().unwrap(), "status"])
        .assert()
        .success();

    assert!(explicit_repo.join("meta").is_dir());
    assert!(explicit_repo.join("chunks").is_dir());
    assert!(!workspace.join("meta").exists());
}

#[test]
fn marker_with_auto_load_false_uses_default_home() {
    let temp = TempDir::new().unwrap();
    let workspace = temp.path().join("workspace");
    let lattice_home = temp.path().join("default-home");
    let xdg_home = temp.path().join("xdg");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&lattice_home).unwrap();
    fs::create_dir_all(&xdg_home).unwrap();

    fs::write(
        workspace.join(LOCAL_REPO_CONFIG_FILE),
        "[repo]\nauto_load = false\n",
    )
    .unwrap();

    lfs_cmd(&lattice_home, &xdg_home)
        .current_dir(&workspace)
        .arg("status")
        .assert()
        .success();

    assert!(lattice_home.join("meta").is_dir());
    assert!(lattice_home.join("chunks").is_dir());
    assert!(!workspace.join("meta").exists());
}
