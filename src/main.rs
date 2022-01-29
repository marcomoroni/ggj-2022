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
    hovered_card: Option<usize>,
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

fn apply_inverse_action<'a, T>(
    action: &Action,
    left_col: &'a mut Vec<T>,
    right_col: &'a mut Vec<T>,
    get_nature: Box<dyn Fn(&T) -> TileNature>,
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
            let index_a = col.iter().position(|n| get_nature(n) == *nature_a).unwrap();
            let index_b = col.iter().position(|n| get_nature(n) == *nature_b).unwrap();
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

fn apply_action<'a, T>(
    action: &Action,
    mut left_col: &'a mut Vec<T>,
    mut right_col: &'a mut Vec<T>,
    get_nature: Box<dyn Fn(&T) -> TileNature>,
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
            let index_a = col.iter().position(|x| get_nature(x) == *nature_a).unwrap();
            let index_b = col.iter().position(|x| get_nature(x) == *nature_b).unwrap();
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
    // If used, store the order in which was used.
    used: Option<usize>,
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

#[derive(Clone, Copy)]
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
                apply_inverse_action(
                    &card_to_apply,
                    &mut build_left_col,
                    &mut build_right_col,
                    Box::new(|x| x.nature),
                );
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
                used: None,
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
                    .with_children(|parent| match card_action {
                        Action::SwapFirstAndLast { side } => {}
                        Action::SwapTwoAdjacent { top, side } => {}
                        Action::SwapTwoNatures {
                            nature_a,
                            nature_b,
                            side,
                        } => {}
                        Action::Cycle {
                            times,
                            direction,
                            side,
                        } => {
                            parent.spawn_bundle(SpriteBundle {
                                transform: Transform {
                                    translation: Vec3::new(
                                        match side {
                                            TileSide::Left => -15.,
                                            TileSide::Right => 15.,
                                        },
                                        0.,
                                        10.,
                                    ),
                                    ..Default::default()
                                },
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(110., 110.)),
                                    ..Default::default()
                                },
                                texture: asset_server.load(match direction {
                                    CycleDirection::Up => "card_cycle_up.png",
                                    CycleDirection::Down => "card_cycle_down.png",
                                }),
                                ..Default::default()
                            });
                        }
                    })
                    .id(),
            });
        }

        *match_state = MatchState::Playing(MatchStatePlaying {
            left_col,
            right_col,
            cards,
            hovered_card: Some(0),
        });
    }
}

fn handle_input(keyboard_input: Res<Input<KeyCode>>, mut match_state: ResMut<MatchState>) {
    if keyboard_input.just_pressed(KeyCode::Left) {
        match match_state.as_mut() {
            MatchState::Playing(match_state) => {
                if let Some(hovered_card) = &mut match_state.hovered_card {
                    *hovered_card = {
                        if *hovered_card == 0 {
                            match_state.cards.len() - 1
                        } else {
                            *hovered_card - 1
                        }
                    };
                }
            }
            _ => (),
        }
    }

    if keyboard_input.just_pressed(KeyCode::Right) {
        match match_state.as_mut() {
            MatchState::Playing(match_state) => {
                if let Some(hovered_card) = &mut match_state.hovered_card {
                    *hovered_card = {
                        if *hovered_card == match_state.cards.len() - 1 {
                            0
                        } else {
                            *hovered_card + 1
                        }
                    };
                }
            }
            _ => (),
        }
    }

    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::Return) {
        match match_state.as_mut() {
            MatchState::Playing(match_state) => {
                if let Some(hovered_card) = &mut match_state.hovered_card {
                    // If card not used.
                    if match_state.cards[*hovered_card].used == None {
                        apply_action(
                            &match_state.cards[*hovered_card].action,
                            &mut match_state.left_col,
                            &mut match_state.right_col,
                            Box::new(|x| x.nature),
                        );

                        // Set as used by also storing its order.
                        match_state.cards[*hovered_card].used = Some(
                            match match_state.cards.iter().filter_map(|x| x.used).max() {
                                Some(i) => i + 1,
                                None => 0,
                            },
                        );
                    }
                }
            }
            _ => (),
        }
    }
}

#[derive(Component)]
struct Cursor;

const CURSOR_Y_POS: f32 = -500.;

fn setup_cursor(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(30., 30.)),
                ..Default::default()
            },
            texture: asset_server.load("cursor.png"),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(Cursor);
}

// on match_state changed
fn update_cursor(
    mut q_cursor: Query<(&mut Transform, &mut Visibility), With<Cursor>>,
    match_state: Res<MatchState>,
) {
    let (mut transform, mut visibility) = q_cursor.single_mut();
    match match_state.as_ref() {
        MatchState::Ready => {
            visibility.is_visible = false;
        }
        MatchState::Playing(match_state) => {
            match match_state.hovered_card {
                Some(i) => {
                    let tot_card_len = CARDS_GAP * ((match_state.cards.len() - 1) as f32);
                    transform.translation = Vec3::new(
                        (tot_card_len / ((match_state.cards.len() - 1) as f32) * (i as f32))
                            - (tot_card_len / 2.),
                        CURSOR_Y_POS,
                        10.,
                    );
                }
                None => {
                    visibility.is_visible = false;
                }
            }
            visibility.is_visible = true;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<StartMatchEvent>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_startup_system(setup_cursor)
        .add_system(start_match)
        .add_system(handle_input)
        .add_system(update_cursor)
        .run();
}
