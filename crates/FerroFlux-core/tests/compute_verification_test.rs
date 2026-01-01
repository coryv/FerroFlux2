use bevy_ecs::prelude::*;
use ferroflux_core::components::compute::ComputeConfig;
use ferroflux_core::components::{Inbox, Outbox, WorkDone};
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::compute::{WasmRuntime, wasm_worker};
use std::collections::VecDeque;

#[test]
fn test_wasm_compute_quickjs() {
    let mut world = World::new();

    // 1. Resources
    world.insert_resource(WasmRuntime::default());
    let store = BlobStore::new();
    world.insert_resource(store.clone());
    world.insert_resource(WorkDone::default());

    // 2. Setup Input Data
    let input_bytes = b"ignored".to_vec();
    let ticket = store
        .check_in(&input_bytes)
        .expect("Failed to check in data");

    // 3. Spawn Compute Entity
    let mut inbox = Inbox {
        queue: VecDeque::new(),
    };
    inbox.queue.push_back(ticket);

    world.spawn((
        ComputeConfig {
            runtime: "simple.wat".to_string(),
            source_code: "".to_string(), // Ignored by simple.wat
            entry_point: "_start".to_string(),
        },
        inbox,
        Outbox::default(),
    ));

    // 4. Run System
    let mut schedule = Schedule::default();
    schedule.add_systems(wasm_worker);
    schedule.run(&mut world);

    // 5. Verify Output
    let mut query = world.query::<&mut Outbox>();
    let mut outbox = query.single_mut(&mut world);

    assert!(
        !outbox.queue.is_empty(),
        "Outbox should have a result ticket"
    );
    let (_port, result_ticket) = outbox.queue.pop_front().unwrap();

    // Claim result
    let result_arc = store.claim(&result_ticket).expect("Failed to claim result");
    let result_str = String::from_utf8(result_arc.to_vec()).unwrap();
    println!("WASM Output: {}", result_str);

    // simple.wat prints "Hello WASM\n"
    // WASI stdout might include newline.
    // We check if it contains "Hello WASM"
    assert!(
        result_str.contains("Hello WASM"),
        "Output should contain 'Hello WASM'"
    );
}

#[test]
fn test_wasm_timeout() {
    let mut world = World::new();

    // 1. Resources
    world.insert_resource(WasmRuntime::default());
    let store = BlobStore::new();
    world.insert_resource(store.clone());
    world.insert_resource(WorkDone::default());

    // 2. Setup Input Data
    let input_bytes = b"{}".to_vec();
    let ticket = store
        .check_in(&input_bytes)
        .expect("Failed to check in data");

    // 3. Spawn Compute Entity with Infinite Loop
    let mut inbox = Inbox {
        queue: VecDeque::new(),
    };
    inbox.queue.push_back(ticket);

    world.spawn((
        ComputeConfig {
            runtime: "loop.wat".to_string(),
            source_code: "".to_string(),
            entry_point: "_start".to_string(),
        },
        inbox,
        Outbox::default(),
    ));

    // 4. Run System
    let mut schedule = Schedule::default();
    schedule.add_systems(wasm_worker);
    schedule.run(&mut world);

    // 5. Verify Output Error
    let mut query = world.query::<&mut Outbox>();
    let mut outbox = query.single_mut(&mut world);

    let (_port, result_ticket) = outbox.queue.pop_front().unwrap();
    let result_arc = store.claim(&result_ticket).expect("Failed to claim result");
    let result_str = String::from_utf8(result_arc.to_vec()).unwrap();
    println!("Timeout Output: {}", result_str);

    assert!(
        result_str.contains("error") || result_str.contains("Runtime Error"),
        "Result should be an error"
    );
}
