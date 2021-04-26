fn main() {
    let _ = env_logger::try_init();

    let mut engine = application::WasiEngine::new().unwrap();

    let modules = &[
        "release/plugin.wasm"
    ];

    let base_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target")
        .join("wasm32-wasi");

    for module in modules {
        let path = base_path.join(module);
        engine.load_module(path.as_path()).unwrap();
    }

    let data = "this is a test";

    engine.run(data.as_bytes()).unwrap();
}
