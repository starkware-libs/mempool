fn main() {
    tonic_build::compile_protos("proto/rpc_network_component_sender.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
