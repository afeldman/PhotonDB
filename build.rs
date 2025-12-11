fn main() {
    // Compile all Cap'n Proto schema files
    capnpc::CompilerCommand::new()
        .src_prefix("proto")
        .file("proto/types.capnp")
        .file("proto/handshake.capnp")
        .file("proto/term.capnp")
        .file("proto/query.capnp")
        .file("proto/response.capnp")
        .run()
        .expect("capnp compilation failed");
}
