use ferroflux_sdk::FerroFluxClient;
use flow_canvas::input::{InputState, Key, ModifiersState, MouseButtons};
use flow_canvas::model::GraphState;
use flow_canvas::render::DrawCommand;
use flow_canvas::{Canvas, CanvasConfig};
use macroquad::prelude as mq;
use uuid::Uuid;
// Use the glam version compatible with FlowCanvas/SDK
// use glam; // Redundant if using macroquad's re-export or if not needed directly

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct NodeData {
    name: String,
}

#[macroquad::main("FerroFlux Playground")]
async fn main() {
    // 1. Initialize Engine (SDK)
    println!("Initializing FerroFlux SDK...");
    let mut client = match FerroFluxClient::<NodeData>::init().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to init SDK: {:?}", e);
            return;
        }
    };
    println!("SDK Initialized.");

    // 2. Initialize Canvas
    let config = CanvasConfig::default();
    let mut canvas = Canvas::new(config);
    let mut graph = GraphState::<NodeData>::default();

    // Add a demo node
    graph.insert_node(flow_canvas::model::Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: Uuid::new_v4(),
        position: glam::Vec2::new(100.0, 100.0),
        size: glam::Vec2::new(200.0, 100.0),
        inputs: vec![],
        outputs: vec![],
        data: NodeData {
            name: "Start Node".into(),
        },
        flags: Default::default(),
        style: None,
    });

    loop {
        let screen_w = mq::screen_width();
        let screen_h = mq::screen_height();
        canvas.update_viewport_size(glam::Vec2::new(screen_w, screen_h));

        // 3. Input Handling
        // Map Macroquad input to FlowCanvas InputState
        let (mx, my) = mq::mouse_position();
        let mouse_pos = glam::Vec2::new(mx, my);
        let wheel = mq::mouse_wheel().1;

        let mut buttons = MouseButtons::default();
        if mq::is_mouse_button_down(mq::MouseButton::Left) {
            buttons.left = true;
        }
        if mq::is_mouse_button_down(mq::MouseButton::Right) {
            buttons.right = true;
        }
        if mq::is_mouse_button_down(mq::MouseButton::Middle) {
            buttons.middle = true;
        }

        let modifiers = ModifiersState {
            ctrl: mq::is_key_down(mq::KeyCode::LeftControl)
                || mq::is_key_down(mq::KeyCode::RightControl),
            shift: mq::is_key_down(mq::KeyCode::LeftShift)
                || mq::is_key_down(mq::KeyCode::RightShift),
            alt: mq::is_key_down(mq::KeyCode::LeftAlt) || mq::is_key_down(mq::KeyCode::RightAlt),
            meta: mq::is_key_down(mq::KeyCode::LeftSuper)
                || mq::is_key_down(mq::KeyCode::RightSuper),
        };

        // Naive key mapping for demo
        let mut pressed_keys = Vec::new();
        if mq::is_key_pressed(mq::KeyCode::Delete) {
            pressed_keys.push(Key::Delete);
        }
        if mq::is_key_pressed(mq::KeyCode::Backspace) {
            pressed_keys.push(Key::Backspace);
        }
        if mq::is_key_pressed(mq::KeyCode::A) {
            pressed_keys.push(Key::A);
        }

        let input = InputState {
            mouse_pos,
            mouse_buttons: buttons,
            scroll_delta: wheel,
            modifiers,
            pressed_keys,
            screen_size: glam::Vec2::new(screen_w, screen_h),
            event_consumed_by_content: false,
        };

        // 4. Update Logic
        if let Err(e) = client.tick().await {
            eprintln!("Tick error: {:?}", e);
        }
        client.sync_events(&mut graph);

        let (draw_list, events) = canvas.update(&input, mq::get_frame_time(), &mut graph);

        for event in events {
            println!("Logic Event: {:?}", event);
            if let flow_canvas::interaction::LogicEvent::Connect { .. } = event {
                let _ = client.compile_and_deploy(&graph).await;
            }
        }

        // 5. Render
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
                    // Fill
                    mq::draw_rectangle(
                        pos.x,
                        pos.y,
                        size.x,
                        size.y,
                        mq::Color::new(color.x, color.y, color.z, color.w),
                    );

                    // Stroke
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
                    // Approximate bezier
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

        mq::next_frame().await
    }
}
