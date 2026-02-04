fn main() {
    let proto = "../services/proto/ipc.proto";
    println!("cargo:rerun-if-changed={}", proto);
    let mut config = prost_build::Config::new();
    config.compile_protos(&[proto], &["../services/proto"]).unwrap();
}
