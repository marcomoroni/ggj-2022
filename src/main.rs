use bevy::prelude::*;
use rand::{distributions::Uniform, prelude::*};

const BACKGROUND_COLOR: Color = Color::rgb(0.05, 0.05, 0.05);
const TILES_LEFT: [&str; 6] = [
    "tile_a_l.png",
    "tile_b_l.png",
    "tile_c_l.png",
    "tile_d_l.png",
    "tile_e_l.png",
    "tile_f_l.png",
];
const TILES_RIGHT: [&str; 6] = [
    "tile_a_r.png",
    "tile_b_r.png",
    "tile_c_r.png",
    "tile_d_r.png",
    "tile_e_r.png",
    "tile_f_r.png",
];

const fn nature_count() -> usize {
    TILES_LEFT.len()
}

enum MatchState {
    Ready,
    Playing {
        left_col: Vec<Entity>,
        right_col: Vec<Entity>,
    },
}

#[derive(Clone, Copy)]
struct TileNature(usize);

#[derive(Clone, Copy)]
enum TileSide {
    Left,
    Right,
}

#[derive(Component, Clone, Copy)]
struct Tile {
    nature: TileNature,
    side: TileSide,
}

struct StartMatchEvent;

fn setup(mut commands: Commands, mut start_match_event: EventWriter<StartMatchEvent>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(MatchState::Ready);
    start_match_event.send(StartMatchEvent);
}

const TILE_POS_X_ABS: f32 = 200.;
const TILE_POS_Y_GAP: f32 = 120.;

// Not a system!
fn spawn_tile(
    side: TileSide,
    nature: TileNature,
    pos_y: f32,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(
                    match side {
                        TileSide::Left => -TILE_POS_X_ABS,
                        TileSide::Right => TILE_POS_X_ABS,
                    },
                    pos_y,
                    0.,
                ),
                ..Default::default()
            },
            sprite: Sprite {
                custom_size: Some(Vec2::new(40., 40.)),
                ..Default::default()
            },
            texture: asset_server.load(match side {
                TileSide::Left => TILES_LEFT[nature.0],
                TileSide::Right => TILES_RIGHT[nature.0],
            }),
            ..Default::default()
        })
        .insert(Tile { nature, side })
        .id()
}

fn start_match(
    mut commands: Commands,
    mut start_match_event: EventReader<StartMatchEvent>,
    asset_server: Res<AssetServer>,
    mut match_state: ResMut<MatchState>,
) {
    for _ in start_match_event.iter() {
        let tiles_count = 4;
        let tiles_order = {
            assert!(tiles_count <= TILES_LEFT.len());
            let mut pool = (0..nature_count()).collect::<Vec<usize>>();
            let mut rng = rand::thread_rng();
            let mut tiles = Vec::new();
            for _ in 0..tiles_count {
                tiles.push(TileNature(pool.swap_remove(rng.gen_range(0, pool.len()))));
            }
            tiles
        };

        let mut left_col = Vec::new();
        let mut right_col = Vec::new();
        let tot_col_height = TILE_POS_Y_GAP * ((tiles_count - 1) as f32);
        for (row, nature) in tiles_order.iter().enumerate() {
            let pos_y = (tot_col_height / ((tiles_count - 1) as f32) * (row as f32))
                - (tot_col_height / 2.);
            left_col.push(spawn_tile(
                TileSide::Left,
                *nature,
                pos_y,
                &mut commands,
                &asset_server,
            ));
            right_col.push(spawn_tile(
                TileSide::Right,
                *nature,
                pos_y,
                &mut commands,
                &asset_server,
            ));
        }

        // Generate cards.
        // ...

        *match_state = MatchState::Playing {
            left_col,
            right_col,
        };
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<StartMatchEvent>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_system(start_match)
        .run();
}
