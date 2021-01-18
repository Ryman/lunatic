use super::types::*;

use anyhow::Result;
use uptown_funk::{host_functions, types, Trap};

use log::trace;
use std::io::{self, IoSlice, IoSliceMut, Read, Write};

lazy_static::lazy_static! {
    static ref ENV : WasiEnv = WasiEnv::env_vars(std::env::vars());
    static ref ARG : WasiEnv = WasiEnv::args(std::env::args());
}

pub struct WasiState {}

impl WasiState {
    pub fn new() -> Self {
        Self {}
    }
}

type ExitCode = super::types::ExitCode<WasiState>;
type Ptr<T> = types::Pointer<WasiState, T>;
type Status = Result<types::Status<WasiState>, Trap>;

#[host_functions(namespace = "wasi_snapshot_preview1")]
impl WasiState {
    fn clock_time_get(&self, _id: u32, _precision: u64) -> (u32, u64) {
        // TODO
        (0, 0)
    }

    fn path_filestat_get(&self, _fd: u32, _flags: u32, _path: &str) -> (u32, u32) {
        // TODO
        (0, 0)
    }

    async fn sched_yield(&self) -> u32 {
        smol::future::yield_now().await;
        0
    }

    fn random_get(&self, _buf: &mut [u8]) -> u32 {
        // TODO
        0
    }

    fn proc_exit(&self, _exit_code: ExitCode) {}

    fn fd_write(&self, fd: u32, ciovs: &[IoSlice<'_>]) -> (u32, u32) {
        match fd {
            // Stdin not supported as write destination
            0 => (WASI_EINVAL, 0),
            1 => {
                let written = io::stdout().write_vectored(ciovs).unwrap();
                (WASI_ESUCCESS, written as u32)
            }
            2 => {
                let written = io::stderr().write_vectored(ciovs).unwrap();
                (WASI_ESUCCESS, written as u32)
            }
            _ => panic!("Unsupported wasi write destination"),
        }
    }

    fn fd_read(&self, fd: u32, iovs: &mut [IoSliceMut<'_>]) -> (u32, u32) {
        match fd {
            // Stdout & stderr not supported as read destination
            1 | 2 => (WASI_EINVAL, 0),
            0 => {
                let written = io::stdin().read_vectored(iovs).unwrap();
                (WASI_ESUCCESS, written as u32)
            }
            _ => panic!("Unsupported wasi read destination"),
        }
    }

    fn path_open(
        &self,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _e: u32,
        _f: i64,
        _g: i64,
        _h: u32,
    ) -> (u32, u32) {
        (0, 0)
    }

    fn fd_close(&self, fd: u32) -> u32 {
        trace!("wasi_snapshot_preview1:fd_close({})", fd);
        WASI_ESUCCESS
    }

    fn fd_prestat_get(&self, _fd: u32, _prestat_ptr: u32) -> u32 {
        WASI_EBADF
    }

    fn fd_prestat_dir_name(&self, _fd: u32, _path: &str) -> u32 {
        WASI_ESUCCESS
    }

    fn args_sizes_get(&self, mut var_count: Ptr<u32>, mut total_bytes: Ptr<u32>) -> Status {
        var_count.set(&ARG.len());
        total_bytes.set(&ARG.total_bytes());
        WasiStatus::Success.into()
    }

    fn args_get(&self, mut args: Ptr<Ptr<u8>>, mut args_buf: Ptr<u8>) -> Status {
        for kv in ARG.iter() {
            args.set(&args_buf);
            args_buf = args_buf
                .copy_slice(&kv)?
                .ok_or_else(|| Trap::new("Reached end of the args buffer"))?;
            args = args
                .next()
                .ok_or_else(|| Trap::new("Reached end of the args pointer buffer"))?;
        }

        WasiStatus::Success.into()
    }

    fn environ_sizes_get(&self, mut var_count: Ptr<u32>, mut total_bytes: Ptr<u32>) -> Status {
        var_count.set(&ENV.len());
        total_bytes.set(&ENV.total_bytes());
        WasiStatus::Success.into()
    }

    fn environ_get(&self, mut environ: Ptr<Ptr<u8>>, mut environ_buf: Ptr<u8>) -> Status {
        for kv in ENV.iter() {
            environ.set(&environ_buf);
            environ_buf = environ_buf
                .copy_slice(&kv)?
                .ok_or_else(|| Trap::new("Reached end of the environment variables buffer"))?;
            environ = environ
                .next()
                .ok_or_else(|| Trap::new("Reached end of the environ var pointer buffer"))?;
        }

        WasiStatus::Success.into()
    }
}
