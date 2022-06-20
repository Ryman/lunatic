use anyhow::Result;
use lunatic_common_api::{get_memory, IntoTrap};
use lunatic_distributed::DistributedProcessState;
use wasmtime::{Caller, Linker, ResourceLimiter, Trap};

pub trait DistributedCtx {
    fn distributed(&self) -> Result<&DistributedProcessState>;
    fn distributed_mut(&mut self) -> Result<&mut DistributedProcessState>;
}

// Register the process APIs to the linker
pub fn register<T>(linker: &mut Linker<T>) -> Result<()>
where
    T: DistributedCtx + Send + ResourceLimiter + 'static,
    for<'a> &'a T: Send,
{
    linker.func_wrap("lunatic::distributed", "nodes_count", nodes_count)?;
    linker.func_wrap("lunatic::distributed", "get_nodes", get_nodes)?;
    linker.func_wrap("lunatic::distributed", "node_id", node_id)?;
    //linker.func_wrap7_async("lunatic::distributed", "spawn", spawn)?;
    Ok(())
}

// Returns count of registered nodes
fn nodes_count<T: DistributedCtx>(caller: Caller<T>) -> u32 {
    caller
        .data()
        .distributed()
        .map(|d| d.control.node_count())
        .unwrap_or(0) as u32
}

// Copy node ids to memory TODO doc
fn get_nodes<T: DistributedCtx>(
    mut caller: Caller<T>,
    nodes_ptr: u32,
    nodes_len: u32,
) -> Result<u32, Trap> {
    let memory = get_memory(&mut caller)?;
    let node_ids = caller
        .data()
        .distributed()
        .map(|d| d.control.node_ids())
        .unwrap_or_else(|_| vec![]);
    memory
        .data_mut(&mut caller)
        .get_mut(
            nodes_ptr as usize
                ..(nodes_ptr as usize + std::mem::size_of::<u64>() * nodes_len as usize),
        )
        .or_trap("lunatic::distributed::get_nodes::memory")?
        .copy_from_slice(unsafe { node_ids.align_to::<u8>().1 });
    Ok(2)
}

// Spawns a new process using the passed in function inside a module as the entry point.
//
// If **link** is not 0, it will link the child and parent processes. The value of the **link**
// argument will be used as the link-tag for the child. This means, if the child traps the parent
// is going to get a signal back with the value used as the tag.
//
// If *config_id* or *module_id* have the value 0, the same module/config is used as in the
// process calling this function.
//
// The function arguments are passed as an array with the following structure:
// [0 byte = type ID; 1..17 bytes = value as u128, ...]
// The type ID follows the WebAssembly binary convention:
//  - 0x7F => i32
//  - 0x7E => i64
//  - 0x7B => v128
// If any other value is used as type ID, this function will trap.
//
// TODO add link and config support
//
// Returns:
// * 0 on success - The ID of the newly created process is written to **id_ptr**
// * 1 on error   - The error ID is written to **id_ptr**
//
// Traps:
// * If the module ID doesn't exist.
// * If the function string is not a valid utf8 string.
// * If the params array is in a wrong format.
// * If any memory outside the guest heap space is referenced.
//#[allow(clippy::too_many_arguments)]
//fn spawn<T>(
//    mut caller: Caller<T>,
//    node_id: u64,
//    module_id: u64,
//    func_str_ptr: u32,
//    func_str_len: u32,
//    params_ptr: u32,
//    params_len: u32,
//    id_ptr: u32,
//) -> Box<dyn Future<Output = Result<u32, Trap>> + Send + '_>
//where
//    T: DistributedCtx + ResourceLimiter + Send + 'static,
//    for<'a> &'a T: Send,
//{
//    Box::new(async move {
//        let state = caller.data_mut();
//        unimplemented!()
//    })
//}

// Returns ID of the node that the current process is running on
fn node_id<T: DistributedCtx>(caller: Caller<T>) -> u64 {
    caller
        .data()
        .distributed()
        .as_ref()
        .map(|d| d.node_id())
        .unwrap_or(0)
}
