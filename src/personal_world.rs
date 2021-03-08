use crate::constants::{CHUNKSIZE, METACHUNKSIZE, METACHUNK_GEN_RANGE, METACHUNK_UNLOAD_RADIUS};
use crate::player::Player;
use crate::positions::{ChunkPos, MetaChunkPos};
use crate::renderer::chunk_render_data::ChunkRenderData;
use crate::renderer::renderer::{resize, Renderer};
use crate::ui::ui::UiRenderer;
use crate::world::world::World;
use crate::world_gen::chunk_gen_thread::ChunkGenThread;
use crate::world_gen::meta_chunk::MetaChunk;
use cgmath::{InnerSpace, Vector3};
use rayon::iter::ParallelIterator;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};
use rayon::prelude::ParallelSliceMut;
use std::cmp::{min, Ordering};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use winit::event::Event;
use winit::event_loop::ControlFlow;
use winit::window::Window;
use winit_window_control::input::input::Input;
use winit_window_control::main_loop::RenderResult;

pub struct PersonalWorld {
    pub world: World,
    pub chunk_render_data: HashMap<ChunkPos, ChunkRenderData>,
    pub player: Player,
    pub chunk_gen_thread: ChunkGenThread,
    pub loading_chunks: HashSet<MetaChunkPos>,
    pub renderer: Renderer,
    pub reload_vertex_load_order: bool,
    pub to_generate: Vec<(f32, ChunkPos)>,
    pub ui: UiRenderer,
}

impl PersonalWorld {
    pub fn new(window: &Window) -> PersonalWorld {
        let renderer = Renderer::new(&window);
        let ui_renderer = UiRenderer::new(window, &renderer);
        PersonalWorld {
            renderer,
            world: World::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32,
            ),
            chunk_render_data: HashMap::new(),
            player: Player::new(),
            chunk_gen_thread: ChunkGenThread::new(),
            loading_chunks: HashSet::new(),
            reload_vertex_load_order: false,
            to_generate: Vec::new(),
            ui: ui_renderer,
        }
    }
    pub fn update(&mut self) {
        self.world.update();
        /*let timer = Instant::now();
        let chunk = self.world.get_chunk(&ChunkPos { x: 2, y: 2, z: 2 });
        if chunk.is_none() {
            return;
        }
        let chunk = chunk.unwrap();
        let main: &[u8] = bytemuck::cast_slice(&chunk.blocks.d);

        self.renderer.wgpu.compute.compute_pass(
            &self.renderer.wgpu.device,
            &self.renderer.wgpu.queue,
            main,
        );
        println!("compute time: {}", timer.elapsed().as_secs_f64());*/
    }
    pub fn on_game_tick(&mut self, dt: f32) {
        self.player.update(&dt, &self.world);
        self.update();
        self.load_generated_chunks();
        self.to_generate = self.vertex_buffers_to_generate();
        if self.player.generated_chunks_for != self.player.position.get_chunk()
            || self.reload_vertex_load_order
        {
            self.on_player_moved_chunks();
            self.player.generated_chunks_for = self.player.position.get_chunk();
            self.reload_vertex_load_order = false;
        }
    }

    pub fn vertex_buffers_to_generate(&self) -> Vec<(f32, ChunkPos)> {
        let mut to_render = Vec::with_capacity(
            self.world.get_all_chunks().len() * METACHUNKSIZE * METACHUNKSIZE * METACHUNKSIZE,
        );
        for (_, meta_chunk) in self.world.get_all_chunks() {
            for (_, pos) in meta_chunk.get_iter() {
                let (should_gen, additional_weight) =
                    self.should_generate_vertex_buffers(pos.clone());
                if should_gen {
                    let distance = pos.get_distance(&self.player.position.get_chunk());
                    to_render.push((distance - additional_weight, pos.clone()));
                }
            }
        }
        to_render.par_sort_unstable_by(|val1, val2| {
            if val1 > val2 {
                return Ordering::Less;
            } else if val2 > val1 {
                return Ordering::Greater;
            }
            return Ordering::Equal;
        });
        return to_render;
    }
    pub fn should_generate_vertex_buffers(&self, pos: ChunkPos) -> (bool, f32) {
        let distance = pos.get_distance(&self.player.position.get_chunk());
        if distance > self.player.render_distance {
            return (false, 0.0);
        }

        if self.world.get_chunk(&pos.get_diff(0, 0, 1)).is_none()
            || self.world.get_chunk(&pos.get_diff(0, 0, -1)).is_none()
            || (self.world.get_chunk(&pos.get_diff(0, 1, 0)).is_none()
                && pos.y + 1 != METACHUNKSIZE as i32)
            || (self.world.get_chunk(&pos.get_diff(0, -1, 0)).is_none() && pos.y - 1 >= 0)
            || self.world.get_chunk(&pos.get_diff(1, 0, 0)).is_none()
            || self.world.get_chunk(&pos.get_diff(-1, 0, 0)).is_none()
        {
            return (false, 0.0);
        }
        if self.chunk_render_data.contains_key(&pos) {
            return (false, 0.0);
        }
        let view_dir = Vector3::new(
            self.player.direction.x,
            self.player.direction.y,
            self.player.direction.z,
        );
        let viewer_pos = Vector3::new(
            self.player.position.x,
            self.player.position.y,
            self.player.position.z,
        );
        let chunk_pos = Vector3::new(
            (pos.x * CHUNKSIZE as i32) as f32,
            (pos.y * CHUNKSIZE as i32) as f32,
            (pos.z * CHUNKSIZE as i32) as f32,
        );
        let difference = viewer_pos - chunk_pos;

        if view_dir.dot(difference) / (view_dir.magnitude() * difference.magnitude()) < -0.5 {
            return (true, 1000.0);
        }
        return (true, 0.0);
    }
    pub fn meta_chunk_should_be_loaded(player: &Player, pos: &MetaChunkPos) -> bool {
        let player_chunk_pos = player.position.get_meta_chunk();
        pos.x <= player_chunk_pos.x + METACHUNK_UNLOAD_RADIUS as i32
            && pos.x >= player_chunk_pos.x - METACHUNK_UNLOAD_RADIUS as i32
            && pos.z <= player_chunk_pos.z + METACHUNK_UNLOAD_RADIUS as i32
            && pos.z >= player_chunk_pos.z - METACHUNK_UNLOAD_RADIUS as i32
    }
    pub fn load_chunk(&mut self, pos: MetaChunkPos) {
        if self.world.chunk_exists_or_generating(&pos) {
            return;
        }
        self.loading_chunks.insert(pos.clone());
        let chunk_request_result = self.chunk_gen_thread.request(pos, self.world.world_seed);
        match chunk_request_result {
            Ok(_) => (),
            Err(e) => println!("error while trying to load A chunk: {}", e),
        }
    }
    pub fn on_player_moved_chunks(&mut self) {
        self.check_chunks_to_generate();
        self.world.filter_chunks(&self.player);
        let player = &self.player;
        self.chunk_render_data
            .retain(|pos, _| MetaChunk::retain_meta_chunk(player, pos.get_meta_chunk_pos()));
    }
    pub fn check_chunks_to_generate(&mut self) {
        let current_chunk = self.player.position.get_meta_chunk();
        let mut to_load = BinaryHeap::new();
        for x in current_chunk.x - METACHUNK_GEN_RANGE as i32 - 1
            ..current_chunk.x + METACHUNK_GEN_RANGE as i32 + 1
        {
            for z in current_chunk.z - METACHUNK_GEN_RANGE as i32 - 1
                ..current_chunk.z + METACHUNK_GEN_RANGE as i32 + 1
            {
                if PersonalWorld::meta_chunk_should_be_loaded(&self.player, &MetaChunkPos { x, z })
                    && !self.loading_chunks.contains(&MetaChunkPos { x, z })
                    && !self
                        .chunk_render_data
                        .contains_key(&MetaChunkPos { x, z }.get_center_pos().get_chunk())
                {
                    let chunk_pos = MetaChunkPos { x, z };
                    to_load.push((
                        (chunk_pos.get_distance_to_object(&self.player.position) * -10f32) as i64,
                        chunk_pos,
                    ));
                }
            }
        }
        while !to_load.is_empty() {
            self.load_chunk(to_load.pop().unwrap().1);
        }
    }
    pub fn check_vertices_to_generate(&mut self) -> i32 {
        if self.to_generate.is_empty() {
            return 0;
        }
        let lag_timer = Instant::now();
        //println!("started generating vertices");
        let starting_size = self.to_generate.len();
        while lag_timer.elapsed().as_secs_f32() < 0.001 && !self.to_generate.is_empty() {
            let len = self.to_generate.len();
            if len > 0 {
                let (_, pos) = &self.to_generate[self.to_generate.len() - 1];
                let data = ChunkRenderData::new(&self.world, &pos, &self.renderer.wgpu.device);
                self.chunk_render_data.insert(pos.clone(), data);
            }
            self.to_generate.remove(self.to_generate.len() - 1);
        }
        /*println!(
            "done generating: {} vertices in: {} sec",
            starting_size - self.to_generate.len(),
            lag_timer.elapsed().as_secs_f32()
        );*/
        return (starting_size - self.to_generate.len()) as i32;
    }
    pub fn load_generated_chunks(&mut self) {
        let message = self.chunk_gen_thread.get();
        match message {
            Ok((chunk, pos)) => {
                self.loading_chunks.remove(&pos);
                self.world.add_chunk(pos, chunk);
                self.reload_vertex_load_order = true;
            }
            Err(_) => return,
        }
    }
    pub fn update_ui_input(&mut self, input: &Input) {
        self.ui.update_input(input);
    }
    pub fn render(&mut self, window: &Window) -> RenderResult {
        let main_pipeline = self.renderer.pipelines.get_mut("main").unwrap();
        main_pipeline.uniforms.update_view_proj(
            &self.player,
            (
                self.renderer.wgpu.size.width,
                self.renderer.wgpu.size.height,
            ),
            self.world.time,
        );
        let render_data = &self.chunk_render_data;
        main_pipeline.set_uniform_buffer(&self.renderer.wgpu.queue, main_pipeline.uniforms);
        match self
            .renderer
            .do_render_pass(render_data, &mut self.ui, window, &self.player)
        {
            Ok(_) => {}
            // Recreate the swap_chain if lost
            Err(wgpu::SwapChainError::Lost) => {
                resize(self.renderer.wgpu.size, &mut self.renderer.wgpu)
            }
            // The system is out of memory, we should probably quit
            Err(wgpu::SwapChainError::OutOfMemory) => return RenderResult::Exit,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }

        return RenderResult::Continue;
    }
}
