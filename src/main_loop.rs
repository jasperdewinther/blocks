use crate::input::input::Input;
use crate::personal_world::PersonalWorld;
use crate::renderer::wgpu::WgpuState;
use crate::ui::UiRenderer;
use std::time::Instant;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub struct MainLoop {
    /*event_loop: EventLoop<()>,
window: Window,
renderer: Renderer,
personal_world: PersonalWorld,*/}

impl MainLoop {
    pub fn new() -> MainLoop {
        return MainLoop {
            /*event_loop,
            window,
            renderer,
            personal_world,*/
        };
    }

    pub fn run(self) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_maximized(true)
            .build(&event_loop)
            .unwrap();
        let mut window_input = Input::new();
        let mut personal_world = PersonalWorld::new(&window);
        let mut world_tick_timer = Instant::now();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    window_input.update_cursor_moved(position);
                }
                WindowEvent::CursorEntered { .. } => {
                    window_input.update_cursor_entered();
                }
                WindowEvent::CursorLeft { .. } => {
                    window_input.update_cursor_left();
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    window_input.update_mouse_input(state, button);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    window_input.update_mouse_wheel(delta);
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    window_input.update_keyboard_input(input, control_flow);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    personal_world.ui = UiRenderer::new(&window, &personal_world.renderer);
                    MainLoop::resize(*physical_size, &mut personal_world.renderer.wgpu);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    personal_world.ui = UiRenderer::new(&window, &personal_world.renderer);
                    MainLoop::resize(**new_inner_size, &mut personal_world.renderer.wgpu);
                }

                _ => {}
            },
            Event::RedrawRequested(_) => {
                personal_world
                    .player
                    .handle_input(&window_input, &(0.01 as f32));
                personal_world.render(control_flow, &window, &event);
                window_input.update();
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {
                personal_world.load_generated_chunks();
                if world_tick_timer.elapsed().as_secs_f32() * 20f32 > 1f32 {
                    personal_world.on_game_tick(0.1);
                    world_tick_timer = Instant::now();
                }
            }
        });

        /*let event_loop = EventLoop::new();
        let window = winit::window::Window::new(&event_loop).unwrap();
        let display = create_display(&event_loop);
        let program = gen_program(&display);
        let mut draw_info = DrawInfo {
            display: display,
            program: program,
            program_start: SystemTime::now(),
            draw_params: gen_draw_params(),
        };
        //let mut ui_renderer = UiRenderer::init(&draw_info);

        info!("generating chunk main");
        let mut chunk_manager = ChunkManager::new(10);
        let mut player = Player::new();
        let mut busy_frame_time = 0f64;
        let mut busy_update_time = 0f64;

        let timer = Instant::now();
        let mut rerender_timer = Instant::now();
        const FRAMERATE: f32 = 60f32;
        let mut update_timer = Instant::now();
        let mut update_times = LinkedList::new();
        for _ in 0..30 {
            update_times.push_back(0f32);
        }
        let mut draw_times = LinkedList::new();
        for _ in 0..30 {
            draw_times.push_back(0f32);
        }
        info!("starting main loop");
        event_loop.run(move |event, _, control_flow| {
            if draw_info.program_start.elapsed().unwrap().as_secs_f64() > 60f64 {
                println!("busy frame time: {}", busy_frame_time);
                println!("busy update time: {}", busy_update_time);
                MainLoop::kill_game_loop(control_flow);
                return;
            }
            MainLoop::event_handler(event, control_flow);

            if update_timer.elapsed().as_millis() > 100 {
                let dt = timer.elapsed().as_secs_f32();
                update_timer = Instant::now();
                MainLoop::on_game_tick(&dt, &mut player, &mut chunk_manager);
                chunk_manager.gen_vertex_buffers(&mut draw_info, &player);
                update_times.pop_front();
                update_times.push_back(update_timer.elapsed().as_secs_f32());
                busy_update_time += update_timer.elapsed().as_secs_f64();
            } else if 1f32 / rerender_timer.elapsed().as_secs_f32() < FRAMERATE {
                let dt = rerender_timer.elapsed().as_secs_f32();
                rerender_timer = Instant::now();
                player.handle_input(&dt);

                MainLoop::on_render(
                    &dt,
                    &update_times,
                    &draw_times,
                    &player,
                    &chunk_manager,
                    &mut draw_info,
                    &mut ui_renderer,
                );
                draw_times.pop_front();
                draw_times.push_back(rerender_timer.elapsed().as_secs_f32());
                busy_frame_time += rerender_timer.elapsed().as_secs_f64();
            }
        });*/
    }
    pub(crate) fn resize(new_size: winit::dpi::PhysicalSize<u32>, wgpu: &mut WgpuState) {
        wgpu.size = new_size;
        wgpu.sc_desc.width = new_size.width;
        wgpu.sc_desc.height = new_size.height;
        wgpu.swap_chain = wgpu.device.create_swap_chain(&wgpu.surface, &wgpu.sc_desc);
    }

    /*pub fn on_render(
        _dt: &f32,
        update_buffer: &LinkedList<f32>,
        draw_buffer: &LinkedList<f32>,
        player: &Player,
        world: &ChunkManager,
        draw_info: &mut DrawInfo,
        ui_renderer: &mut UiRenderer,
    ) {
        let mut average_update = 0f32;
        let mut longest_update = 0f32;
        for i in update_buffer.iter() {
            if i.clone() > longest_update {
                longest_update = i.clone();
            }
            average_update += i.clone();
        }
        average_update = average_update / update_buffer.len() as f32;

        let mut average_draw = 0f32;
        let mut longest_draw = 0f32;
        for i in draw_buffer.iter() {
            if i.clone() > longest_draw {
                longest_draw = i.clone();
            }
            average_draw += i.clone();
        }
        average_draw = average_draw / draw_buffer.len() as f32;

        let mut target = draw_info.display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 0.0), 1.0);
        world.render_chunks(draw_info, &mut target, &player);

        let text = vec![
            format!("long up: {}", longest_update.to_string()),
            format!("ave up: {}", average_update.to_string()),
            format!("long dr: {}", longest_draw.to_string()),
            format!("ave dr: {}", average_draw.to_string()),
            format!("total vertex buffers: {}", world.count_vertex_buffers()),
            format!("total chunks: {}", world.count_chunks()),
            format!(
                "total vertex buffers drawn: {}",
                world.count_vertex_buffers_in_range(&player)
            ),
            format!("total vertices: {}", world.count_vertices()),
            format!(
                "x: {} y: {} z: {}",
                player.position.x as i32, player.position.y as i32, player.position.z as i32
            ),
        ];
        let draw_result = ui_renderer.draw(&draw_info, &text, &mut target);
        match draw_result {
            Ok(_) => (),
            Err(e) => println!("error when drawing ui: {}", e),
        }

        target.finish().unwrap();
    }*/

    /*pub fn event_handler(event: Event<()>, control_flow: &mut ControlFlow) {
        *control_flow = match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,

                // Redraw the triangle when the window is resized.
                glutin::event::WindowEvent::Resized(..) => glutin::event_loop::ControlFlow::Poll,
                _ => glutin::event_loop::ControlFlow::Poll,
            },
            _ => glutin::event_loop::ControlFlow::Poll,
        };
    }
    pub fn kill_game_loop(control_flow: &mut ControlFlow) {
        *control_flow = glutin::event_loop::ControlFlow::Exit;
    }*/
}
