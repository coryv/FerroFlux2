use flow_canvas::input::{InputState, Key, ModifiersState, MouseButtons};
use flow_canvas::model::GraphState;
use flow_canvas::render::DrawCommand;
use flow_canvas::{Canvas, CanvasConfig};
use macroquad::prelude as mq;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use uuid::Uuid;

// We need to import the SDK types to send them over channels,
// but we WON'T use the async Client directly on the main thread.
use ferroflux_sdk::FerroFluxClient;

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct NodeData {
    name: String,
}

// -- Messages --

enum BackendMsg {
    Deploy(GraphState<NodeData>),
    // Add other commands here
}

enum FrontendMsg {
    InitSuccess,
    InitError(String),
    // Telemetry(String), // Removed unused variant
}

#[macroquad::main("FerroFlux Playground")]
async fn main() {
    // 1. Setup Channels
    let (to_backend, from_frontend_rx): (Sender<BackendMsg>, Receiver<BackendMsg>) = channel();
    let (to_frontend, from_backend_rx): (Sender<FrontendMsg>, Receiver<FrontendMsg>) = channel();

    // 2. Spawn Backend Thread
    // This thread hosts the Tokio Runtime and the FerroFlux SDK.
    thread::spawn(move || {
        // Create runtime
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime");

        rt.block_on(async move {
            println!("[Backend] Initializing SDK...");
            let mut client = match FerroFluxClient::<NodeData>::init().await {
                Ok(c) => {
                    let _ = to_frontend.send(FrontendMsg::InitSuccess);
                    c
                }
                Err(e) => {
                    let _ = to_frontend.send(FrontendMsg::InitError(format!("{:?}", e)));
                    return;
                }
            };
            println!("[Backend] SDK Ready.");

            // Backend Loop
            loop {
                // 1. Check for messages from UI (Non-blocking)
                match from_frontend_rx.try_recv() {
                    Ok(msg) => match msg {
                        BackendMsg::Deploy(graph) => {
                            println!("[Backend] Deploying graph...");
                            if let Err(e) = client.compile_and_deploy(&graph).await {
                                eprintln!("[Backend] Deploy failed: {:?}", e);
                            }
                        }
                    },
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => break, // UI closed
                }

                // 2. Tick Engine
                if let Err(e) = client.tick().await {
                    eprintln!("[Backend] Tick error: {:?}", e);
                }

                // 3. Process Engine Events -> Send to UI
                // We need to intercept events from client.event_rx manually or expose a helper.
                // For now, let's just use the client's internal sync helper but catch the output?
                // Actually, `client.sync_events` modifies a GraphState reference.
                // Since GraphState is on the UI thread, we can't share it easily without Mutex.
                //
                // Alternative: The backend should "forward" events to the frontend.
                // But `FerroFluxClient` swallows events into `tracing` logs in `sync_events`.
                // For this demo, we'll assume the backend just runs the logic.
                // Real implementation would forward `SystemEvent`s via `to_frontend`.

                // Sleep a tiny bit to prevent 100% CPU on backend loop
                tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
            }
        });
    });

    // 3. Initialize UI State
    let config = CanvasConfig::default();
    let mut canvas = Canvas::new(config);
    let mut graph = GraphState::<NodeData>::default();
    let mut sdk_ready = false;
    let mut error_screen: Option<String> = None;

    // Add demo node immediately so we have something to see
    // Add demo node immediately so we have something to see
    {
        let node_uuid = Uuid::new_v4();
        let node_id = graph.nodes.insert_with_key(|key| flow_canvas::model::Node {
            id: key,
            uuid: node_uuid,
            position: glam::Vec2::new(100.0, 100.0),
            size: glam::Vec2::new(160.0, 80.0),
            inputs: vec![],
            outputs: vec![],
            data: NodeData {
                name: "Start Node".into(),
            },
            flags: Default::default(),
            style: None,
        });
        graph.uuid_index.insert(node_uuid, node_id);

        // Add some ports
        let input_port = graph.ports.insert_with_key(|key| flow_canvas::model::Port {
            id: key,
            node: node_id,
        });
        let output_port = graph.ports.insert_with_key(|key| flow_canvas::model::Port {
            id: key,
            node: node_id,
        });

        if let Some(node) = graph.nodes.get_mut(node_id) {
            node.inputs.push(input_port);
            node.outputs.push(output_port);
        }
    }

    println!("[Frontend] Starting Loop...");

    loop {
        let screen_w = mq::screen_width();
        let screen_h = mq::screen_height();

        // 4. Poll Backend Messages
        while let Ok(msg) = from_backend_rx.try_recv() {
            match msg {
                FrontendMsg::InitSuccess => {
                    println!("[Frontend] Connected to SDK.");
                    sdk_ready = true;
                }
                FrontendMsg::InitError(e) => {
                    error_screen = Some(e);
                } // FrontendMsg::Telemetry matched here previously, removed.
            }
        }

        if let Some(err) = &error_screen {
            mq::clear_background(mq::RED);
            mq::draw_text("FATAL SDK ERROR", 20.0, 40.0, 50.0, mq::WHITE);
            mq::draw_text(err, 20.0, 100.0, 30.0, mq::WHITE);
            mq::next_frame().await;
            continue;
        }

        // 5. Update Canvas
        canvas.update_viewport_size(glam::Vec2::new(screen_w, screen_h));

        // Input
        let (mx, my) = mq::mouse_position();
        let input = InputState {
            mouse_pos: glam::Vec2::new(mx, my),
            mouse_buttons: MouseButtons {
                left: mq::is_mouse_button_down(mq::MouseButton::Left),
                right: mq::is_mouse_button_down(mq::MouseButton::Right),
                middle: mq::is_mouse_button_down(mq::MouseButton::Middle),
            },
            scroll_delta: mq::mouse_wheel().1,
            modifiers: ModifiersState {
                ctrl: mq::is_key_down(mq::KeyCode::LeftControl)
                    || mq::is_key_down(mq::KeyCode::RightControl),
                shift: mq::is_key_down(mq::KeyCode::LeftShift)
                    || mq::is_key_down(mq::KeyCode::RightShift),
                alt: mq::is_key_down(mq::KeyCode::LeftAlt)
                    || mq::is_key_down(mq::KeyCode::RightAlt),
                meta: mq::is_key_down(mq::KeyCode::LeftSuper)
                    || mq::is_key_down(mq::KeyCode::RightSuper),
            },
            pressed_keys: {
                let mut keys = Vec::new();
                if mq::is_key_pressed(mq::KeyCode::Delete) {
                    keys.push(Key::Delete);
                }
                if mq::is_key_pressed(mq::KeyCode::Backspace) {
                    keys.push(Key::Backspace);
                }
                if mq::is_key_pressed(mq::KeyCode::A) {
                    keys.push(Key::A);
                }
                keys
            },
            screen_size: glam::Vec2::new(screen_w, screen_h),
            event_consumed_by_content: false,
        };

        // Handle "Add Node" shortcut (A) manually for this playground
        if mq::is_key_pressed(mq::KeyCode::A) && !input.modifiers.ctrl && !input.modifiers.meta {
            // 0. Convert Mouse Screen -> World
            let world_mouse = canvas.view.screen_to_world(glam::Vec2::new(mx, my));

            // 1. Create Node (initially empty ports)
            let node_uuid = Uuid::new_v4();
            let count = graph.nodes.len();
            let name = if count % 2 == 0 { "Source" } else { "Process" };

            // Minor jitter to prevent perfect stacking if spamming, but keep near cursor
            let offset = (count as f32 % 5.0) * 5.0;
            let pos = world_mouse + glam::Vec2::new(offset, offset);

            // High contrast style
            let style = flow_canvas::config::NodeStyle {
                color: if count % 2 == 0 {
                    glam::Vec4::new(0.2, 0.4, 0.6, 1.0) // Blueish
                } else {
                    glam::Vec4::new(0.6, 0.4, 0.2, 1.0) // Orangeish
                },
                border_color: glam::Vec4::new(0.9, 0.9, 0.9, 1.0),
                text_color: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
            };

            let node_id = graph.nodes.insert_with_key(|key| flow_canvas::model::Node {
                id: key,
                uuid: node_uuid,
                position: pos,
                size: glam::Vec2::new(160.0, 80.0),
                inputs: vec![],
                outputs: vec![],
                data: NodeData { name: name.into() },
                flags: Default::default(),
                style: Some(style),
            });
            graph.uuid_index.insert(node_uuid, node_id);

            // CRITICAL: Must add to draw_order or it will be invisible and unclickable!
            graph.draw_order.push(node_id);

            // 2. Create Ports
            let input_port = graph.ports.insert_with_key(|key| flow_canvas::model::Port {
                id: key,
                node: node_id,
            });
            let output_port = graph.ports.insert_with_key(|key| flow_canvas::model::Port {
                id: key,
                node: node_id,
            });

            // 3. Update Node with Ports
            if let Some(node) = graph.nodes.get_mut(node_id) {
                node.inputs.push(input_port);
                node.outputs.push(output_port);
            }

            println!("[Playground] Added Node: {} at World {:?}", name, pos);
        }

        let (draw_list, events) = canvas.update(&input, mq::get_frame_time(), &mut graph);

        // 6. Handle Logic Events -> Send to Backend
        for event in events {
            println!("[Playground] Event: {:?}", event); // LOG ALL EVENTS
            match event {
                flow_canvas::interaction::LogicEvent::Connect { from, to } => {
                    println!("[Playground] Connecting {:?} -> {:?}", from, to);
                    // 1. Update Local Graph State (frontend visual)
                    // Check for duplicate?
                    let exists = graph
                        .connections
                        .values()
                        .any(|c| c.from == from && c.to == to);
                    if !exists {
                        graph.connections.insert(flow_canvas::model::Connection {
                            from,
                            to,
                            style: flow_canvas::model::WireStyle::Cubic,
                            visual_style: None,
                        });

                        // 2. Send to Backend
                        if sdk_ready {
                            let _ = to_backend.send(BackendMsg::Deploy(graph.clone()));
                        }
                    }
                }
                flow_canvas::interaction::LogicEvent::DeleteSelection => {
                    // Identify valid selected nodes (collect to avoid borrow checker)
                    let selected_ids: Vec<flow_canvas::model::NodeId> = graph
                        .nodes
                        .iter()
                        .filter(|(_, n)| n.flags.contains(flow_canvas::model::NodeFlags::SELECTED))
                        .map(|(id, _)| id)
                        .collect();

                    if !selected_ids.is_empty() {
                        for id in &selected_ids {
                            // A. Cleanup Ports & Connections associated with this node
                            if let Some(node) = graph.nodes.get(*id) {
                                // Collect connection IDs to remove (Connections where From OR To matches node's ports)
                                // This is O(C) per node, which is fine for playground.
                                let bad_conns: Vec<flow_canvas::model::ConnectionId> = graph
                                    .connections
                                    .iter()
                                    .filter(|(_, c)| {
                                        node.inputs.contains(&c.to)
                                            || node.outputs.contains(&c.from)
                                    })
                                    .map(|(cid, _)| cid)
                                    .collect();

                                for cid in bad_conns {
                                    graph.connections.remove(cid);
                                }

                                // Remove Ports
                                for port_id in &node.inputs {
                                    graph.ports.remove(*port_id);
                                }
                                for port_id in &node.outputs {
                                    graph.ports.remove(*port_id);
                                }
                            }

                            // B. Remove Node (updates UUID map)
                            graph.remove_node(*id);

                            // C. Remove from Draw Order
                            if let Some(pos) = graph.draw_order.iter().position(|&x| x == *id) {
                                graph.draw_order.remove(pos);
                            }
                        }

                        println!("[Playground] Deleted {} nodes", selected_ids.len());

                        // Sync with backend
                        if sdk_ready {
                            let _ = to_backend.send(BackendMsg::Deploy(graph.clone()));
                        }
                    }
                }
                _ => {}
            }
        }

        // 7. Render
        mq::clear_background(mq::DARKGRAY);
        for cmd in draw_list {
            match cmd {
                DrawCommand::Rect {
                    pos,
                    size,
                    color,
                    stroke_width,
                    stroke_color,
                    ..
                } => {
                    mq::draw_rectangle(
                        pos.x,
                        pos.y,
                        size.x,
                        size.y,
                        mq::Color::new(color.x, color.y, color.z, color.w),
                    );
                    if let Some(sc) = stroke_color {
                        mq::draw_rectangle_lines(
                            pos.x,
                            pos.y,
                            size.x,
                            size.y,
                            stroke_width,
                            mq::Color::new(sc.x, sc.y, sc.z, sc.w),
                        );
                    }
                }
                DrawCommand::Line {
                    start,
                    end,
                    color,
                    width,
                } => {
                    mq::draw_line(
                        start.x,
                        start.y,
                        end.x,
                        end.y,
                        width,
                        mq::Color::new(color.x, color.y, color.z, color.w),
                    );
                }
                DrawCommand::Bezier {
                    start,
                    end,
                    color,
                    width,
                    ..
                } => {
                    mq::draw_line(
                        start.x,
                        start.y,
                        end.x,
                        end.y,
                        width,
                        mq::Color::new(color.x, color.y, color.z, color.w),
                    );
                }
                DrawCommand::Text {
                    pos,
                    text,
                    color,
                    size,
                } => {
                    mq::draw_text(
                        &text,
                        pos.x,
                        pos.y,
                        size,
                        mq::Color::new(color.x, color.y, color.z, color.w),
                    );
                }
            }
        }

        // DEBUG: Draw Mouse Cursor alignment check
        mq::draw_circle(mx, my, 5.0, mq::RED);
        mq::draw_text(
            &format!("Mouse: {:.1}, {:.1}", mx, my),
            mx + 10.0,
            my,
            20.0,
            mq::YELLOW,
        );

        // Overlay status
        if !sdk_ready {
            mq::draw_text("Connecting to Engine...", 10.0, 30.0, 20.0, mq::YELLOW);
        } else {
            mq::draw_text(
                &format!("Nodes: {}", graph.nodes.len()),
                10.0,
                30.0,
                20.0,
                mq::GREEN,
            );
        }

        mq::next_frame().await
    }
}
