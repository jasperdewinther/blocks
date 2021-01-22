use crate::constants::{METACHUNK_GEN_RANGE, METACHUNK_UNLOAD_RADIUS};
use crate::main_loop::MainLoop;
use crate::player::Player;
use crate::positions::{ChunkPos, MetaChunkPos};
use crate::renderer::chunk_render_data::ChunkRenderData;
use crate::renderer::renderer::Renderer;
use crate::world::World;
use crate::world_gen::chunk_gen_thread::ChunkGenThread;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};
use std::sync::Mutex;
use wgpu::Device;

pub struct PersonalWorld {
    pub world: World,
    pub chunk_render_data: HashMap<ChunkPos, ChunkRenderData>,
    pub player: Player,
    pub chunk_gen_thread: ChunkGenThread,
    pub loading_chunks: HashSet<MetaChunkPos>,
}

impl PersonalWorld {
    pub fn new() -> PersonalWorld {
        PersonalWorld {
            world: World::new(1),
            chunk_render_data: HashMap::new(),
            player: Player::new(),
            chunk_gen_thread: ChunkGenThread::new(),
            loading_chunks: HashSet::new(),
        }
    }
    pub fn update(&mut self, renderer: &Renderer) {
        let chunks = &self.world.chunks;
        for (pos, chunk) in chunks {
            println!("started generating vertices for: {:?}", &pos);
            let data = chunk.generate_vertex_buffers(&renderer.wgpu.device);
            self.chunk_render_data.extend(data.into_iter());
            println!("done generating vertices for: {:?}", &pos);
        }
    }
    pub fn vertex_buffers_to_generate(&self) -> BTreeMap<i32, ChunkPos> {
        let to_render = Mutex::new(BTreeMap::new());
        for (_, meta_chunk) in &self.world.chunks {
            meta_chunk.for_each(|_, pos| {
                if self.should_generate_vertex_buffers(pos.clone()) {
                    let distance = pos.get_distance(&self.player.position.get_chunk());
                    to_render
                        .lock()
                        .unwrap()
                        .insert((distance * 10000f32) as i32, pos.clone());
                }
            });
        }
        return to_render.into_inner().unwrap();
    }
    pub fn should_generate_vertex_buffers(&self, pos: ChunkPos) -> bool {
        let distance = pos.get_distance(&self.player.position.get_chunk());
        if distance > self.player.render_distance {
            return false;
        }
        return true;
    }
    pub fn meta_chunk_should_be_loaded(player: &Player, pos: &MetaChunkPos) -> bool {
        let player_chunk_pos = player.position.get_meta_chunk();
        pos.x <= player_chunk_pos.x + METACHUNK_UNLOAD_RADIUS as i32
            && pos.x >= player_chunk_pos.x - METACHUNK_UNLOAD_RADIUS as i32
            && pos.z <= player_chunk_pos.z + METACHUNK_UNLOAD_RADIUS as i32
            && pos.z >= player_chunk_pos.z - METACHUNK_UNLOAD_RADIUS as i32
    }
    pub fn load_chunk(&mut self, pos: MetaChunkPos) {
        if self.loading_chunks.contains(&pos) {
            return;
        }
        self.loading_chunks.insert(pos.clone());
        let chunk_request_result = self.chunk_gen_thread.request(pos, self.world.world_seed);
        match chunk_request_result {
            Ok(_) => (),
            Err(e) => println!("error while trying to load a chunk: {}", e),
        }
    }
    pub fn on_player_moved_chunks(&mut self) {
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
                {
                    let chunk_pos = MetaChunkPos { x, z };
                    to_load.push((
                        (chunk_pos.get_distance_to_object(&self.player.position) * 10f32) as i64
                            * -1,
                        chunk_pos,
                    ));
                }
            }
        }
        while !to_load.is_empty() {
            self.load_chunk(to_load.pop().unwrap().1);
        }
        let player = &self.player;
        self.world
            .chunks
            .retain(|pos, _| PersonalWorld::meta_chunk_should_be_loaded(&player, pos));
        self.player.generated_chunks_for = self.player.position.get_chunk();
    }
    pub fn load_generated_chunks(&mut self) {
        let message = self.chunk_gen_thread.get();
        match message {
            Ok((chunk, pos)) => {
                self.loading_chunks.remove(&pos);
                self.world.chunks.insert(pos, chunk);
            }
            Err(_) => return,
        }
    }
}
