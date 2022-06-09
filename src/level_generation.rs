use std::hash::Hash;

use super::*;
use bevy::prelude::*;
use bevy::utils::HashSet;
use rand::{rngs::SmallRng, seq::SliceRandom as _};

const CHUNK_SIZE: f32 = 3000.0;

pub struct LevelGenerator {
    world_seed: u64,
    generated_chunks: HashSet<Chunk>,
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chunk(i32, i32);

impl Chunk {
    fn chunk_with_surrounding(pos: Vec3) -> impl Iterator<Item = Chunk> {
        let x = (pos.x / CHUNK_SIZE) as i32;
        let y = (pos.y / CHUNK_SIZE) as i32;

        [
            Chunk(x, y),
            Chunk(x - 1, y + 1),
            Chunk(x, y + 1),
            Chunk(x + 1, y + 1),
            Chunk(x + 1, y),
            Chunk(x + 1, y - 1),
            Chunk(x, y - 1),
            Chunk(x - 1, y - 1),
            Chunk(x - 1, y),
        ]
        .into_iter()
    }

    fn seed(&self) -> u64 {
        let mut hash = self.0 as u64;
        hash ^= (self.1 as u64) << 16;
        hash ^= (self.1 as u64) >> 16;
        hash
    }
}

impl LevelGenerator {
    pub fn new(world_seed: u64) -> Self {
        LevelGenerator {
            world_seed,
            generated_chunks: Default::default(),
        }
    }

    fn generate_chunk(&mut self, chunk: Chunk, cmd: &mut Commands, materials: &GameMaterials) {
        let mut chunk_rng = SmallRng::seed_from_u64(chunk.seed() ^ self.world_seed);
        if self.generated_chunks.contains(&chunk) {
            return;
        }

        let offset = Vec2::new(
            (chunk.0 as f32 * CHUNK_SIZE) - (CHUNK_SIZE * 0.5),
            (chunk.1 as f32 * CHUNK_SIZE) - (CHUNK_SIZE * 0.5),
        );

        // stars
        for position in CellDistribution::with_rng(&mut chunk_rng, CHUNK_SIZE, 150.) {
            cmd.spawn_bundle(SpriteBundle {
                transform: Transform::from_translation((position + offset).extend(0.)),
                texture: materials.star.clone().into(),
                ..Default::default()
            })
            .insert(chunk.clone());
        }

        // nebulas
        let nebula_positions: Vec<Vec2> =
            CellDistribution::with_rng(&mut chunk_rng, CHUNK_SIZE, 1000.).collect();
        for position in nebula_positions {
            let rotation = Quat::from_rotation_z((TAU / 4.0) * chunk_rng.gen_range(0..3) as f32);

            let nebula = materials.nebulas.choose(&mut chunk_rng).cloned().unwrap();

            cmd.spawn_bundle(SpriteBundle {
                transform: Transform {
                    translation: (position + offset).extend(5.),
                    rotation,
                    scale: Vec3::splat(4.0),
                },
                texture: nebula.into(),

                ..Default::default()
            })
            .insert(chunk.clone());
        }

        self.generated_chunks.insert(chunk);
    }
}

pub fn generate_background(
    mut cmd: Commands,
    materials: Res<GameMaterials>,
    mut generator: ResMut<LevelGenerator>,
    explorers: Query<&GlobalTransform, With<ChunkExplorer>>,
    mut chunk_to_generate: Local<HashSet<Chunk>>,
) {
    for explorer in explorers.iter() {
        let chunks = Chunk::chunk_with_surrounding(explorer.translation);
        chunk_to_generate.extend(chunks);
    }

    for chunk in chunk_to_generate.drain() {
        generator.generate_chunk(chunk, &mut cmd, &materials);
    }
}

pub struct CleanupTimer(Timer);

impl Default for CleanupTimer {
    fn default() -> Self {
        CleanupTimer(Timer::from_seconds(1., true))
    }
}

pub fn cleanup_chunks(
    mut cmd: Commands,
    mut timer: Local<CleanupTimer>,
    time: Res<Time>,
    generator: Res<LevelGenerator>,
    chunk_entities: Query<(Entity, &Chunk)>,
    explorers: Query<&GlobalTransform, With<ChunkExplorer>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let chunk_to_cleanup: HashSet<_> = generator
            .generated_chunks
            .iter()
            .filter(|c| {
                let pos = Vec3::new(c.0 as f32 * CHUNK_SIZE, c.1 as f32 * CHUNK_SIZE, 0.0);
                explorers
                    .iter()
                    .all(|explorer| explorer.translation.distance_squared(pos) > (9000.0 * 9000.0))
            })
            .collect();

        for (entity, chunk) in chunk_entities.iter() {
            if chunk_to_cleanup.contains(chunk) {
                cmd.entity(entity).despawn();
            }
        }
    }
}

#[derive(Component)]
pub struct ChunkExplorer;

pub struct CellDistribution<'r, R> {
    current: usize,
    cols: usize,
    cell_size: f32,
    rng: &'r mut R,
}

impl<'r, R> CellDistribution<'r, R> {
    fn with_rng(rng: &'r mut R, size: f32, cell_size: f32) -> Self {
        CellDistribution {
            current: 0,
            cols: (size / cell_size) as usize,
            cell_size,
            rng,
        }
    }
}

impl<R> Iterator for CellDistribution<'_, R>
where
    R: rand::Rng,
{
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.cols * self.cols {
            return None;
        }

        let row = self.current / self.cols;
        let col = self.current % self.cols;

        let row_offset = self.rng.gen_range(0f32..self.cell_size);
        let col_offset = self.rng.gen_range(0f32..self.cell_size);

        self.current += 1;

        Some(Vec2::new(
            row as f32 * self.cell_size + row_offset,
            col as f32 * self.cell_size + col_offset,
        ))
    }
}
