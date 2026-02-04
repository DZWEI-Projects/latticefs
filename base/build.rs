use std::path::Path;
use std::process::Command;

fn main() {
    let proto = "../services/proto/ipc.proto";
    println!("cargo:rerun-if-changed={}", proto);

    if !protoc_available() {
        println!("cargo:warning=protoc not found; using pre-generated IPC bindings");
        return;
    }

    let out_dir = Path::new("src/ipc");
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        println!("cargo:warning=failed to create IPC out dir: {}", err);
        return;
    }

    let mut config = prost_build::Config::new();
    config.out_dir(out_dir);
    if let Err(err) = config.compile_protos(&[proto], &["../services/proto"]) {
        println!("cargo:warning=failed to compile IPC proto: {}", err);
        return;
    }

    let generated = out_dir.join("latticefs.ipc.rs");
    let target = out_dir.join("proto.rs");
    if generated.exists() {
        if let Err(err) = std::fs::rename(&generated, &target) {
            println!("cargo:warning=failed to move IPC bindings: {}", err);
        }
    }
}

fn protoc_available() -> bool {
    if let Ok(path) = std::env::var("PROTOC") {
        return Path::new(&path).exists();
    }

    Command::new("protoc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
