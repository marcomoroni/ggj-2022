use bevy::prelude::*;
use rand::prelude::*;

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

const CARDS_GAP: f32 = 140.;

enum MatchState {
    Ready,
    Playing(MatchStatePlaying),
}

struct MatchStatePlaying {
    left_col: Vec<TileData>,
    right_col: Vec<TileData>,
    cards: Vec<CardData>,
}

#[derive(Clone, Copy)]
struct TileData {
    id: Entity,
    nature: TileNature,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct TileNature(usize);

#[derive(Clone, Copy)]
enum TileSide {
    Left,
    Right,
}

#[derive(Component, Clone, Copy)]
struct Tile;

#[derive(Clone, Copy)]
enum CycleDirection {
    Up,
    Down,
}

#[derive(Clone, Copy)]
enum Action {
    SwapFirstAndLast {
        side: TileSide,
    },
    SwapTwoAdjacent {
        top: usize,
        side: TileSide,
    },
    SwapTwoNatures {
        nature_a: TileNature,
        nature_b: TileNature,
        side: TileSide,
    },
    Cycle {
        times: i32,
        direction: CycleDirection,
        side: TileSide,
    },
}

fn apply_inverse_action<'a>(
    action: &Action,
    left_col: &'a mut Vec<BuildingTileData>,
    right_col: &'a mut Vec<BuildingTileData>,
) {
    match action {
        Action::SwapFirstAndLast { side } => {
            let col = match side {
                TileSide::Left => left_col,
                TileSide::Right => right_col,
            };
            let col_len = col.len();
            col.swap(0, col_len - 1);
        }
        Action::SwapTwoAdjacent { top, side } => {
            let bottom = top + 1;
            let col = match side {
                TileSide::Left => left_col,
                TileSide::Right => right_col,
            };
            col.swap(*top, bottom);
        }
        Action::SwapTwoNatures {
            nature_a,
            nature_b,
            side,
        } => {
            let col = match side {
                TileSide::Left => left_col,
                TileSide::Right => right_col,
            };
            let index_a = col.iter().position(|n| n.nature == *nature_a).unwrap();
            let index_b = col.iter().position(|n| n.nature == *nature_b).unwrap();
            col.swap(index_a, index_b);
        }
        Action::Cycle {
            times,
            direction,
            side,
        } => {
            let col = match side {
                TileSide::Left => left_col,
                TileSide::Right => right_col,
            };
            match direction {
                CycleDirection::Up => {
                    col.rotate_right(*times as usize);
                }
                CycleDirection::Down => {
                    col.rotate_left(*times as usize);
                }
            }
        }
    }
}

fn apply_action<'a>(
    action: &Action,
    mut left_col: &'a mut Vec<BuildingTileData>,
    mut right_col: &'a mut Vec<BuildingTileData>,
) {
    match action {
        Action::SwapFirstAndLast { side } => {
            let col = match side {
                TileSide::Left => &mut left_col,
                TileSide::Right => &mut right_col,
            };
            let col_len = col.len();
            col.swap(0, col_len - 1);
        }
        Action::SwapTwoAdjacent { top, side } => {
            let bottom = top + 1;
            let col = match side {
                TileSide::Left => &mut left_col,
                TileSide::Right => &mut right_col,
            };
            col.swap(*top, bottom);
        }
        Action::SwapTwoNatures {
            nature_a,
            nature_b,
            side,
        } => {
            let col = match side {
                TileSide::Left => &mut left_col,
                TileSide::Right => &mut right_col,
            };
            let index_a = col.iter().position(|x| x.nature == *nature_a).unwrap();
            let index_b = col.iter().position(|x| x.nature == *nature_b).unwrap();
            col.swap(index_a, index_b);
        }
        Action::Cycle {
            times,
            direction,
            side,
        } => {
            let col = match side {
                TileSide::Left => &mut left_col,
                TileSide::Right => &mut right_col,
            };
            match direction {
                CycleDirection::Up => {
                    col.rotate_left(*times as usize);
                }
                CycleDirection::Down => {
                    col.rotate_right(*times as usize);
                }
            }
        }
    }
}

#[derive(Component)]
struct Card;

struct CardData {
    action: Action,
    id: Entity,
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
        .insert(Tile)
        .id()
}

fn rand_tile_side(rng: &mut ThreadRng) -> TileSide {
    match rng.gen_range(0usize, 2usize) {
        0 => TileSide::Left,
        1 => TileSide::Right,
        _ => unreachable!(),
    }
}

struct BuildingTileData {
    id: Option<Entity>,
    nature: TileNature,
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

        let mut build_left_col = Vec::new();
        let mut build_right_col = Vec::new();
        for nature in tiles_order.iter() {
            build_left_col.push(BuildingTileData {
                nature: *nature,
                id: None,
            });
            build_right_col.push(BuildingTileData {
                nature: *nature,
                id: None,
            });
        }

        // Generate cards.
        let card_count = 5;
        let card_actions = {
            let mut rng = rand::thread_rng();
            let mut cards = Vec::new();
            for _ in 0..card_count {
                cards.push(match rng.gen_range(0usize, 4usize) {
                    0 => Action::SwapFirstAndLast {
                        side: rand_tile_side(&mut rng),
                    },
                    1 => Action::SwapTwoAdjacent {
                        top: rng.gen_range(0, tiles_order.len() - 1),
                        side: rand_tile_side(&mut rng),
                    },
                    2 => {
                        let mut pool = tiles_order.clone();
                        let nature_a = pool.swap_remove(rng.gen_range(0, pool.len()));
                        let nature_b = pool.swap_remove(rng.gen_range(0, pool.len()));
                        Action::SwapTwoNatures {
                            nature_a,
                            nature_b,
                            side: rand_tile_side(&mut rng),
                        }
                    }
                    3 => Action::Cycle {
                        times: rng.gen_range(1, 4),
                        direction: match rng.gen_range(0usize, 2usize) {
                            0 => CycleDirection::Up,
                            1 => CycleDirection::Down,
                            _ => unreachable!(),
                        },
                        side: rand_tile_side(&mut rng),
                    },
                    _ => unreachable!(),
                })
            }
            cards
        };

        // Apply some inverse cards_effect.
        let applied_card_count = 3;
        assert!(applied_card_count <= card_count);
        {
            let mut rng = rand::thread_rng();
            let mut cards_to_apply_pool = card_actions.clone();
            for _ in 0..applied_card_count {
                let card_to_apply =
                    cards_to_apply_pool.swap_remove(rng.gen_range(0, cards_to_apply_pool.len()));
                apply_inverse_action(&card_to_apply, &mut build_left_col, &mut build_right_col);
            }
        }

        let mut left_col = Vec::new();
        let mut right_col = Vec::new();
        let tot_col_height = TILE_POS_Y_GAP * ((tiles_count - 1) as f32);
        for (i, (l, r)) in build_left_col
            .iter()
            .zip(build_right_col.iter())
            .enumerate()
        {
            let pos_y =
                (tot_col_height / ((tiles_count - 1) as f32) * (i as f32)) - (tot_col_height / 2.);
            left_col.push(TileData {
                id: spawn_tile(
                    TileSide::Left,
                    l.nature,
                    pos_y,
                    &mut commands,
                    &asset_server,
                ),
                nature: l.nature,
            });
            right_col.push(TileData {
                id: spawn_tile(
                    TileSide::Right,
                    r.nature,
                    pos_y,
                    &mut commands,
                    &asset_server,
                ),
                nature: r.nature,
            });
        }

        // Spawn cards.
        let tot_card_len = CARDS_GAP * ((card_count - 1) as f32);
        let mut cards = Vec::new();
        for (i, card_action) in card_actions.iter().enumerate() {
            cards.push(CardData {
                action: card_action.clone(),
                id: commands
                    .spawn_bundle(SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(
                                (tot_card_len / ((card_count - 1) as f32) * (i as f32))
                                    - (tot_card_len / 2.),
                                -400.,
                                0.,
                            ),
                            ..Default::default()
                        },
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(160., 160.)),
                            ..Default::default()
                        },
                        texture: asset_server.load("card_bg.png"),
                        ..Default::default()
                    })
                    .insert(Card)
                    .with_children(|parent| {
                        // ...
                    })
                    .id(),
            });
        }

        *match_state = MatchState::Playing(MatchStatePlaying {
            left_col,
            right_col,
            cards,
        });
    }
}

fn handle_input(keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Left) {}

    if keyboard_input.just_pressed(KeyCode::Right) {}

    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::Return) {
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<StartMatchEvent>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_system(start_match)
        .add_system(handle_input)
        .run();
}
