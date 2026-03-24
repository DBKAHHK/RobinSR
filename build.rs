fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("find protoc");
    std::env::set_var("PROTOC", protoc);

    println!("cargo:rerun-if-changed=StarRail.proto");

    prost_build::Config::new()
        .compile_protos(&["StarRail.proto"], &["."])
        .expect("compile StarRail.proto");
}
