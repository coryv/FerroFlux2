use crate::components::compute::ComputeConfig;
use crate::components::{Inbox, Outbox, WorkDone};
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;
use wasi_common::pipe::ReadPipe;
use wasi_common::sync::WasiCtxBuilder;
use wasmtime::{Config, Engine, Linker, Module, Store};

/// Runtime configuration and cache for the WASM engine.
///
/// Holds the expensive `wasmtime::Engine` and caches compiled `Modules` to avoid
/// recompilation latency on every execution (which can be 100ms+).
#[derive(Resource)]
pub struct WasmRuntime {
    pub engine: Engine,
    pub module_cache: Mutex<HashMap<String, Module>>, // Path -> Module
}

impl Default for WasmRuntime {
    fn default() -> Self {
        let mut config = Config::new();
        config.consume_fuel(true); // Enable fuel for timeout
        let engine = Engine::new(&config).expect("Failed to create Wasmtime engine");
        Self {
            engine,
            module_cache: Mutex::new(HashMap::new()),
        }
    }
}

/// System: Wasm Compute Worker
///
/// **Role**: Executes untrusted code snippets in a secure sandbox.
///
/// **Mental Model**:
/// 1. **Poll**: Checks `Inbox` for tasks.
/// 2. **Load**: Retrieves the full payload from `BlobStore`.
/// 3. **Instantiate**: Spins up a fresh WASI environment for *each* execution to ensure isolation.
///    - Stdin receives the JSON payload.
///    - Stdout is captured as the result.
/// 4. **Execute**: Runs the "adapter" WASM (e.g., QuickJS or Python interpreter) which loads the user's script.
/// 5. **Persist**: Stores the captured Stdout back to `BlobStore` and pushes a ticket to `Outbox`.
///
/// **Performance Note**: Modules are cached in `WasmRuntime`, but instances are created per-request.
/// This incurs a ~2ms overhead per execution, which is acceptable for this architecture.
#[tracing::instrument(skip(_commands, query, wasm_runtime, blob_store, work_done))]
pub fn wasm_worker(
    _commands: Commands,
    mut query: Query<(Entity, &mut Inbox, &mut Outbox, &ComputeConfig)>,
    wasm_runtime: Option<Res<WasmRuntime>>,
    blob_store: Option<Res<BlobStore>>,
    mut work_done: ResMut<WorkDone>,
) {
    let runtime = match wasm_runtime {
        Some(r) => r,
        None => return,
    };

    let store = match blob_store {
        Some(s) => s,
        None => return,
    };

    for (_entity, mut inbox, mut outbox, config) in query.iter_mut() {
        if inbox.queue.is_empty() {
            continue;
        }

        while let Some(ticket) = inbox.queue.pop_front() {
            work_done.0 = true;

            // 1. Claim Data from BlobStore
            let input_bytes_arc = match store.claim(&ticket) {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!(ticket_id = %ticket.id, error = %e, "Failed to claim ticket for compute");
                    continue;
                }
            };

            // Convert to JSON Value for processing, or kept as bytes?
            // WASI stdin usually takes bytes.
            // But we assume the data is JSON for our "Protocol".
            // Let's pass the raw bytes to stdin.
            let input_data = input_bytes_arc;

            // 2. Prepare Runtime Adapter Path
            let adapter_filename = match config.runtime.as_str() {
                "python" | "python-3.11" => Some("python.wasm"),
                "js" | "js-quickjs" | "quickjs" => Some("quickjs.wasm"),
                "simple.wat" => Some("simple.wat"),
                "loop.wat" => Some("loop.wat"),
                _ => None,
            };

            let adapter_filename = if let Some(f) = adapter_filename {
                f
            } else {
                tracing::error!("Unsupported runtime: {}", config.runtime);
                let err_msg = format!("{{\"error\": \"Unsupported runtime: {}\"}}", config.runtime);
                if let Ok(t) =
                    store.check_in_with_metadata(&err_msg.into_bytes(), ticket.metadata.clone())
                {
                    outbox.queue.push_back((None, t));
                }
                continue;
            };

            let adapter_path = format!("assets/runtimes/{}", adapter_filename);

            // 3. Load/Get Module
            let module_res = {
                let mut cache = runtime.module_cache.lock().unwrap();
                if let Some(m) = cache.get(&adapter_path) {
                    Ok(m.clone())
                } else {
                    match Module::from_file(&runtime.engine, &adapter_path) {
                        Ok(m) => {
                            cache.insert(adapter_path.clone(), m.clone());
                            Ok(m)
                        }
                        Err(e) => Err(e),
                    }
                }
            };

            let mut final_result_bytes = Vec::new();

            match module_res {
                Ok(module) => {
                    // 4. Setup WASI Context
                    // We pipe the Input Payload -> Stdin
                    // We capture Stdout -> Output Payload
                    // We capture Stderr -> Logging (TODO: Emit as Log events)
                    let stdin = ReadPipe::from(input_data);
                    let stdout = wasi_common::pipe::WritePipe::new_in_memory();
                    let stderr = wasi_common::pipe::WritePipe::new_in_memory();

                    let wasi = WasiCtxBuilder::new()
                        .stdin(Box::new(stdin))
                        .stdout(Box::new(stdout.clone()))
                        .stderr(Box::new(stderr.clone()))
                        .args(&[
                            "runtime".to_string(),
                            "-e".to_string(),
                            config.source_code.clone(),
                        ]) // Arg0 is program name
                        .expect("Failed to set args")
                        .build();

                    let mut wasm_store = Store::new(&runtime.engine, wasi);
                    // 2 Seconds timeout roughly = 100M fuel instructions
                    // Ensure 'add_fuel' is available.
                    if let Err(e) = wasm_store.set_fuel(100_000_000) {
                        tracing::warn!(error = %e, "Failed to set fuel for WASM store");
                    }

                    let mut linker = Linker::new(&runtime.engine);
                    wasi_common::sync::add_to_linker(&mut linker, |s| s).unwrap();

                    // Stub missing socket functions if adapter requires them (e.g. WasmEdge builds)
                    // sock_open(pool_fd, af, socktype) -> errno
                    // 76 = ENOSYS (Not supported)
                    // Force override sock_accept as WasmEdge expects 2 args, WASI has 3.
                    // This must be done unconditionally to shadow the wasi-common implementation.
                    linker
                        .func_wrap(
                            "wasi_snapshot_preview1",
                            "sock_accept",
                            |_: i32, _: i32, _: i32| -> i32 { 76 },
                        )
                        .ok();

                    // Stub other missing functions
                    let socket_funcs = [
                        ("sock_open", 3),
                        ("sock_bind", 3),
                        ("sock_listen", 2),
                        // sock_accept is handled above
                        ("sock_connect", 3),
                        ("sock_recv", 6),
                        ("sock_send", 4),
                        ("sock_shutdown", 2),
                        ("sock_setsockopt", 5),
                        ("sock_getsockopt", 5),
                        ("sock_getpeername", 3),
                        ("sock_getsockname", 3),
                        ("sock_getlocaladdr", 3),
                        ("poll_oneoff", 4),
                        ("sock_getpeeraddr", 3),
                        ("sock_getsockaddr", 3),
                        ("sock_getaddrinfo", 8),
                    ];

                    for (name, _) in socket_funcs {
                        if linker
                            .get(&mut wasm_store, "wasi_snapshot_preview1", name)
                            .is_none()
                        {
                            match name {
                                "sock_open" | "sock_bind" | "sock_connect" | "sock_getpeername"
                                | "sock_getsockname" | "sock_getlocaladdr" | "sock_getpeeraddr"
                                | "sock_getsockaddr" => {
                                    linker
                                        .func_wrap(
                                            "wasi_snapshot_preview1",
                                            name,
                                            |_: i32, _: i32, _: i32| -> i32 { 76 },
                                        )
                                        .ok();
                                }
                                "sock_listen" | "sock_shutdown" => {
                                    linker
                                        .func_wrap(
                                            "wasi_snapshot_preview1",
                                            name,
                                            |_: i32, _: i32| -> i32 { 76 },
                                        )
                                        .ok();
                                }
                                "sock_send" | "poll_oneoff" => {
                                    linker
                                        .func_wrap(
                                            "wasi_snapshot_preview1",
                                            name,
                                            |_: i32, _: i32, _: i32, _: i32| -> i32 { 76 },
                                        )
                                        .ok();
                                }
                                "sock_setsockopt" | "sock_getsockopt" => {
                                    linker
                                        .func_wrap(
                                            "wasi_snapshot_preview1",
                                            name,
                                            |_: i32, _: i32, _: i32, _: i32, _: i32| -> i32 { 76 },
                                        )
                                        .ok();
                                }
                                "sock_recv" => {
                                    linker
                                        .func_wrap(
                                            "wasi_snapshot_preview1",
                                            name,
                                            |_: i32,
                                             _: i32,
                                             _: i32,
                                             _: i32,
                                             _: i32,
                                             _: i32|
                                             -> i32 {
                                                76
                                            },
                                        )
                                        .ok();
                                }
                                "sock_getaddrinfo" => {
                                    linker
                                        .func_wrap(
                                            "wasi_snapshot_preview1",
                                            name,
                                            |_: i32,
                                             _: i32,
                                             _: i32,
                                             _: i32,
                                             _: i32,
                                             _: i32,
                                             _: i32,
                                             _: i32|
                                             -> i32 {
                                                76
                                            },
                                        )
                                        .ok();
                                }
                                _ => {}
                            }
                        }
                    }

                    // 5. Instantiate and Run
                    match linker.instantiate(&mut wasm_store, &module) {
                        Ok(instance) => {
                            let start_func =
                                instance.get_typed_func::<(), ()>(&mut wasm_store, "_start");
                            match start_func {
                                Ok(func) => {
                                    match func.call(&mut wasm_store, ()) {
                                        Ok(_) => {
                                            // Success
                                            drop(wasm_store);
                                            // Extract stdout
                                            if let Ok(locked) = stdout.try_into_inner() {
                                                final_result_bytes = locked.into_inner();
                                            }
                                        }
                                        Err(e) => {
                                            // Runtime Error
                                            let err_msg =
                                                format!("{{\"error\": \"Runtime Error: {}\"}}", e);
                                            final_result_bytes = err_msg.into_bytes();
                                        }
                                    }
                                }
                                Err(e) => {
                                    let err_msg =
                                        format!("{{\"error\": \"Missing _start: {}\"}}", e);
                                    final_result_bytes = err_msg.into_bytes();
                                }
                            }
                        }
                        Err(e) => {
                            let err_msg = format!("{{\"error\": \"Instantiation Failed: {}\"}}", e);
                            final_result_bytes = err_msg.into_bytes();
                        }
                    }
                }
                Err(e) => {
                    let err_msg = format!("{{\"error\": \"Adapter not found: {}\"}}", e);
                    final_result_bytes = err_msg.into_bytes();
                }
            }

            // 6. Check In Result
            match store.check_in_with_metadata(&final_result_bytes, ticket.metadata.clone()) {
                Ok(new_ticket) => {
                    tracing::debug!(ticket_id = %new_ticket.id, "Compute result checked in");
                    outbox.queue.push_back((None, new_ticket));
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to check in compute result");
                }
            }
        }
    }
}
