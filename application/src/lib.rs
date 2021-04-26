#![deny(improper_ctypes, improper_ctypes_definitions)]
mod errors;

pub use errors::Error;

use log::*;
use common::*;
use wasmer::*;
use wasmer_wasi::{WasiEnv, WasiState};
use std::path::Path;

#[derive(Debug, Clone, WasmerEnv)]
pub struct HostMemory {
    _memory: Memory,
}

fn host_memory(_env: &HostMemory) -> WasmPtr<u8, Array> {
    WasmPtr::new(MEMORY_START as _)
}

pub struct ModuleInfo {
    instance: Instance,
    plugin: Plugin,
    memory: Memory,
    run_func: NativeFunc<(u32, WasmPtr<u8, Array>, u32), WasmPtr<u8, Array>>,
}

pub struct WasiEngine {
    store: Store,
    memory: Memory,
    modules: Vec<ModuleInfo>,
}

impl WasiEngine {
    pub fn new() -> Result<Self, Error> {
        let store = Store::default();
        let memory = Memory::new(&store, MemoryType::new(1, None, false))?;

        Ok(Self {
            store: store,
            memory: memory,
            modules: Vec::default(),
        })
    }

    pub fn load_module(&mut self, path: &Path) -> Result<(), Error> {
        info!("Loading module {:?}", path);
        let module = Module::from_file(&self.store, path).map_err(|e| Error::custom(format!("{:?}", e)))?;

        let wasi_version = wasmer_wasi::get_wasi_version(&module, false).expect("Could not detect WASI ABI in Wasm module");
        let mut state_builder = WasiState::new("wasmer-test");
        let state = state_builder.build()?;
        let wasi_env = WasiEnv::new(state);
        let import_object = wasmer_wasi::generate_import_object_from_env(&self.store, wasi_env, wasi_version);

        let memory = HostMemory {
            _memory: self.memory.clone(),
        };
        let plugin_import_object = imports! {
            "env" => {
                "host_memory" => Function::new_native_with_env(&self.store, memory, host_memory),
            }
        };
        let import_object = import_object.chain_back(plugin_import_object);
        let instance = Instance::new(&module, &import_object)?;

        let memory = instance.exports.get_memory("memory")?.clone();

        let func: NativeFunc<(), WasmPtr<u8, Array>> = instance.exports.get_native_function("new")?;
        let addr = func.call()?;
        let plugin: common::Plugin = read(&memory, addr)?;
        info!("Loaded plugin {:?}", plugin);

        let func = instance.exports.get_native_function("run")?;

        let module = ModuleInfo {
            instance: instance,
            plugin: plugin,
            memory: memory,
            run_func: func,
        };
        self.modules.push(module);
        Ok(())
    }

    pub fn run(&self, data: &[u8]) -> Result<Option<String>, Error> {
        let addr: WasmPtr<u8, Array> = WasmPtr::new(MEMORY_START as _);
        let mem = addr.deref(&self.memory, 0, data.len() as _).ok_or_else(|| Error::InvalidOffset)?;
        for (idx, cell) in mem.iter().enumerate() {
            cell.set(data[idx]);
        }
        let data_len = data.len() as u32;
        for module in self.modules.iter() {
            let addr = module.run_func.call(module.plugin.address.into(), addr.into(), data_len.into())?;
            let r: Option<String> = read(&module.memory, addr)?;
            if r.is_some() {
                return Ok(r);
            }
        }
        return Ok(None);
    }
}

fn read<T>(memory: &Memory, address: WasmPtr<u8, Array>) -> Result<T, Error> where T: for<'de> serde::Deserialize<'de> {
    let data = address.deref(memory, 0 as _, LEN_SIZE as _).ok_or_else(|| Error::InvalidOffset)?;
    let mut data_len = [0u8; LEN_SIZE];
    for (idx, cell) in data.iter().enumerate() {
        data_len[idx] = cell.get();
    }
    let data_len = u32::from_ne_bytes(data_len) as usize;
    info!("Data marked as {}B of memory", data_len);
    if data_len > 0 {
        let data = address.deref(memory, LEN_SIZE as _, data_len as _)
            .ok_or_else(|| Error::InvalidOffset)?;
        let data: Vec<_> = data.iter().map(|c| c.get()).collect();
        info!("Deserializing {}B of memory", data.len());
        bincode::deserialize(data.as_slice()).map_err(Error::from)
    } else {
        Err(Error::NullPtr)
    }
}
