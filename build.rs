use protoc_grpcio;

fn main() {
    let proto_root = "proto";
    println!("cargo:rerun-if-changed={}", proto_root);
    protoc_grpcio::compile_grpc_protos(&["api.proto"], &[proto_root], "src/apiproto", None)
        .expect("Failed to compile gRPC definitions!");
}
