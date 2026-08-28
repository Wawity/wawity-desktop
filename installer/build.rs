fn main() {
    println!("cargo:rerun-if-env-changed=WAWITY_VARIANT");
    embed_resource::compile("installer.rc", embed_resource::NONE);
}
