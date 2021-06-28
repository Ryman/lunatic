use std::any::type_name;
use std::collections::HashMap;
use std::fmt::Debug;

use tokio::sync::mpsc::UnboundedReceiver;
use wasmtime::Module;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

use crate::process::environment::{EnvConfig, Environment};
use crate::process::message::Message;
use crate::process::ProcessHandle;

/// The internal state of each Process.
///
/// Host functions will share one state.
pub(crate) struct State {
    // Messages sent to the process
    pub(crate) mailbox: UnboundedReceiver<Message>,
    // Errors belonging to the process
    pub(crate) errors: HashMapId<anyhow::Error>,
    // Resources
    pub(crate) resources: Resources,
    // WASI
    pub(crate) wasi: WasiCtx,
    // The module that is being added to the environment.
    // This makes it accessible inside of plugins that run on the module before it's compiled.
    pub(crate) module_loaded: Option<Vec<u8>>,
}

impl State {
    pub fn new(mailbox: UnboundedReceiver<Message>) -> Self {
        Self {
            mailbox,
            errors: HashMapId::new(),
            resources: Resources::default(),
            // TODO: Inherit args & envs
            wasi: WasiCtxBuilder::new().inherit_stdio().build(),
            module_loaded: None,
        }
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("process", &self.resources)
            .finish()
    }
}

#[derive(Default, Debug)]
pub(crate) struct Resources {
    pub(crate) configs: HashMapId<EnvConfig>,
    pub(crate) environments: HashMapId<Environment>,
    pub(crate) modules: HashMapId<Module>,
    pub(crate) processes: HashMapId<ProcessHandle>,
}

/// HashMap wrapper with incremental ID (u64) assignment.
pub struct HashMapId<T> {
    id_seed: u64,
    store: HashMap<u64, T>,
}

impl<T> HashMapId<T> {
    pub fn new() -> Self {
        Self {
            id_seed: 0,
            store: HashMap::new(),
        }
    }

    pub fn add(&mut self, item: T) -> u64 {
        let id = self.id_seed;
        self.store.insert(id, item);
        self.id_seed += 1;
        id
    }

    pub fn remove(&mut self, id: u64) -> Option<T> {
        self.store.remove(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut T> {
        self.store.get_mut(&id)
    }

    pub fn get(&self, id: u64) -> Option<&T> {
        self.store.get(&id)
    }
}

impl<T> Default for HashMapId<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for HashMapId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashMapId")
            .field("id_seed", &self.id_seed)
            .field("type", &type_name::<T>())
            .finish()
    }
}
