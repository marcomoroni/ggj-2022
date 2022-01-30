mod physics;

use std::f32::consts::PI;

use bevy::{prelude::*, text::Text2dSize};
use physics::spring::SpringSimulation;
use rand::prelude::*;

const BACKGROUND_COLOR: Color = Color::rgb(0.866666666667, 0.8, 0.686274509804);
const TILES_LEFT: [&str; 8] = [
    "tile_a_l.png",
    "tile_b_l.png",
    "tile_c_l.png",
    "tile_d_l.png",
    "tile_e_l.png",
    "tile_f_l.png",
    "tile_g_l.png",
    "tile_h_l.png",
];
const TILES_RIGHT: [&str; 8] = [
    "tile_a_r.png",
    "tile_b_r.png",
    "tile_c_r.png",
    "tile_d_r.png",
    "tile_e_r.png",
    "tile_f_r.png",
    "tile_g_r.png",
    "tile_h_r.png",
];

const fn nature_count() -> usize {
    TILES_LEFT.len()
}

const CARDS_GAP: f32 = 180.;

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
                CycleDirection::Down => {
                    col.rotate_right(*times as usize);
                }
                CycleDirection::Up => {
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
                CycleDirection::Down => {
                    col.rotate_left(*times as usize);
                }
                CycleDirection::Up => {
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
const TILE_POS_Y_GAP: f32 = 170.;

fn tiles_layout_poss(gap: f32, count: usize) -> (Vec<Vec2>, Vec<Vec2>) {
    let y_adjust = 150.;
    let tot_col_height = gap * ((count - 1) as f32);
    let mut l = Vec::new();
    let mut r = Vec::new();
    for i in 0..count {
        let pos_y =
            (tot_col_height / ((count - 1) as f32) * (i as f32)) - (tot_col_height / 2.) + y_adjust;
        l.push(Vec2::new(-TILE_POS_X_ABS, pos_y));
        r.push(Vec2::new(TILE_POS_X_ABS, pos_y));
    }
    (l, r)
}

// Not a system!
fn spawn_tile(
    side: TileSide,
    nature: TileNature,
    pos: Vec2,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(pos.x, pos.y, 0.),
                ..Default::default()
            },
            sprite: Sprite {
                custom_size: Some(Vec2::new(150., 150.)),
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
    mut event_update_cards_style: EventWriter<UpdateCardsStyle>,
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
                        // times: rng.gen_range(1, 4),
                        times: 1,
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
        let (tiles_pos_left, tiles_pos_right) = tiles_layout_poss(TILE_POS_Y_GAP, tiles_count);
        for (i, (l, r)) in build_left_col
            .iter()
            .zip(build_right_col.iter())
            .enumerate()
        {
            left_col.push(TileData {
                id: spawn_tile(
                    TileSide::Left,
                    l.nature,
                    tiles_pos_left[i],
                    &mut commands,
                    &asset_server,
                ),
                nature: l.nature,
            });
            right_col.push(TileData {
                id: spawn_tile(
                    TileSide::Right,
                    r.nature,
                    tiles_pos_right[i],
                    &mut commands,
                    &asset_server,
                ),
                nature: r.nature,
            });
        }

        // Spawn cards.
        let tot_card_len = CARDS_GAP * ((card_count - 1) as f32);
        let mut cards = Vec::new();
        let card_size = 270.;
        let card_illustration_full_col_gap = 40.;
        let card_illustration_full_col_height =
            card_illustration_full_col_gap * ((card_count - 1) as f32);
        let card_illustration_full_col_pos = (0..card_count)
            .map(|i| {
                card_illustration_full_col_height / ((card_count - 1) as f32) * (i as f32)
                    - (card_illustration_full_col_height / 2.)
            })
            .collect::<Vec<f32>>();
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
                                -370.,
                                0.,
                            ),

                            ..Default::default()
                        },
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(card_size, card_size)),
                            ..Default::default()
                        },
                        texture: asset_server.load("card_bg.png"),
                        ..Default::default()
                    })
                    .insert(Card)
                    .with_children(|parent| {
                        let card_as_text = match card_action {
                            Action::SwapFirstAndLast { side } => {
                                let sprite = Sprite {
                                    custom_size: Some(Vec2::new(30., 30.)),
                                    ..Default::default()
                                };
                                let pos_x = match side {
                                    TileSide::Left => -15.,
                                    TileSide::Right => 15.,
                                };

                                for i in 0..card_count {
                                    parent.spawn_bundle(SpriteBundle {
                                        transform: Transform {
                                            translation: Vec3::new(
                                                pos_x,
                                                card_illustration_full_col_pos[i],
                                                10.,
                                            ),
                                            ..Default::default()
                                        },
                                        sprite: sprite.clone(),
                                        texture: asset_server.load(match side {
                                            TileSide::Left => {
                                                if i == 0 || i == card_count - 1 {
                                                    "tile_any_l.png"
                                                } else {
                                                    "tile_empty_l.png"
                                                }
                                            }
                                            TileSide::Right => {
                                                if i == 0 || i == card_count - 1 {
                                                    "tile_any_r.png"
                                                } else {
                                                    "tile_empty_r.png"
                                                }
                                            }
                                        }),
                                        ..Default::default()
                                    });
                                }

                                parent.spawn_bundle(SpriteBundle {
                                    transform: Transform {
                                        translation: Vec3::new(
                                            match side {
                                                TileSide::Left => -41.,
                                                TileSide::Right => 41.,
                                            },
                                            0.,
                                            10.,
                                        ),

                                        ..Default::default()
                                    },
                                    sprite: Sprite {
                                        custom_size: Some(Vec2::new(card_size, card_size)),
                                        ..Default::default()
                                    },
                                    texture: asset_server.load("swap_arrow.png"),
                                    ..Default::default()
                                });

                                format!(
                                    "Swap first and last - {}",
                                    match side {
                                        TileSide::Left => "left",
                                        TileSide::Right => "right",
                                    }
                                )
                            }
                            Action::SwapTwoAdjacent { top, side } => {
                                let sprite = Sprite {
                                    custom_size: Some(Vec2::new(30., 30.)),
                                    ..Default::default()
                                };
                                let pos_x = match side {
                                    TileSide::Left => -15.,
                                    TileSide::Right => 15.,
                                };

                                for i in 0..card_count {
                                    parent.spawn_bundle(SpriteBundle {
                                        transform: Transform {
                                            translation: Vec3::new(
                                                pos_x,
                                                card_illustration_full_col_pos[i],
                                                10.,
                                            ),
                                            ..Default::default()
                                        },
                                        sprite: sprite.clone(),
                                        texture: asset_server.load(match side {
                                            TileSide::Left => {
                                                if i == *top || i == *top + 1 {
                                                    "tile_any_l.png"
                                                } else {
                                                    "tile_empty_l.png"
                                                }
                                            }
                                            TileSide::Right => {
                                                if i == *top || i == *top + 1 {
                                                    "tile_any_r.png"
                                                } else {
                                                    "tile_empty_r.png"
                                                }
                                            }
                                        }),
                                        ..Default::default()
                                    });
                                }

                                parent.spawn_bundle(SpriteBundle {
                                    transform: Transform {
                                        translation: Vec3::new(
                                            match side {
                                                TileSide::Left => -41.,
                                                TileSide::Right => 41.,
                                            },
                                            0.,
                                            10.,
                                        ),

                                        ..Default::default()
                                    },
                                    sprite: Sprite {
                                        custom_size: Some(Vec2::new(card_size, card_size)),
                                        ..Default::default()
                                    },
                                    texture: asset_server.load("swap_arrow.png"),
                                    ..Default::default()
                                });

                                format!(
                                    "Swap {} and {} - {}",
                                    top,
                                    top + 1,
                                    match side {
                                        TileSide::Left => "left",
                                        TileSide::Right => "right",
                                    }
                                )
                            }
                            Action::SwapTwoNatures {
                                nature_a,
                                nature_b,
                                side,
                            } => {
                                let tile_size = 38.;
                                let pos_y_abs = 30.;

                                parent.spawn_bundle(SpriteBundle {
                                    transform: Transform {
                                        translation: Vec3::new(
                                            match side {
                                                TileSide::Left => -tile_size / 2.,
                                                TileSide::Right => tile_size / 2.,
                                            },
                                            -pos_y_abs,
                                            10.,
                                        ),
                                        ..Default::default()
                                    },
                                    sprite: Sprite {
                                        custom_size: Some(Vec2::new(tile_size, tile_size)),
                                        ..Default::default()
                                    },
                                    texture: asset_server.load(match side {
                                        TileSide::Left => TILES_LEFT[nature_a.0],
                                        TileSide::Right => TILES_RIGHT[nature_a.0],
                                    }),
                                    ..Default::default()
                                });

                                parent.spawn_bundle(SpriteBundle {
                                    transform: Transform {
                                        translation: Vec3::new(
                                            match side {
                                                TileSide::Left => -tile_size / 2.,
                                                TileSide::Right => tile_size / 2.,
                                            },
                                            pos_y_abs,
                                            10.,
                                        ),
                                        ..Default::default()
                                    },
                                    sprite: Sprite {
                                        custom_size: Some(Vec2::new(tile_size, tile_size)),
                                        ..Default::default()
                                    },
                                    texture: asset_server.load(match side {
                                        TileSide::Left => TILES_LEFT[nature_b.0],
                                        TileSide::Right => TILES_RIGHT[nature_b.0],
                                    }),
                                    ..Default::default()
                                });

                                parent.spawn_bundle(SpriteBundle {
                                    transform: Transform {
                                        translation: Vec3::new(
                                            match side {
                                                TileSide::Left => -41.,
                                                TileSide::Right => 41.,
                                            },
                                            0.,
                                            10.,
                                        ),

                                        ..Default::default()
                                    },
                                    sprite: Sprite {
                                        custom_size: Some(Vec2::new(card_size, card_size)),
                                        ..Default::default()
                                    },
                                    texture: asset_server.load("swap_arrow.png"),
                                    ..Default::default()
                                });

                                format!(
                                    "Swap {:?} and {:?} - {}",
                                    nature_a,
                                    nature_b,
                                    match side {
                                        TileSide::Left => "left",
                                        TileSide::Right => "right",
                                    }
                                )
                            }
                            Action::Cycle {
                                times,
                                direction,
                                side,
                            } => {
                                let sprite = Sprite {
                                    custom_size: Some(Vec2::new(30., 30.)),
                                    ..Default::default()
                                };
                                let texture = asset_server.load(match side {
                                    TileSide::Left => "tile_any_l.png",
                                    TileSide::Right => "tile_any_r.png",
                                });
                                let pos_x = match side {
                                    TileSide::Left => -15.,
                                    TileSide::Right => 15.,
                                };

                                for i in 0..card_count {
                                    parent.spawn_bundle(SpriteBundle {
                                        transform: Transform {
                                            translation: Vec3::new(
                                                pos_x,
                                                card_illustration_full_col_pos[i],
                                                10.,
                                            ),
                                            ..Default::default()
                                        },
                                        sprite: sprite.clone(),
                                        texture: texture.clone(),
                                        ..Default::default()
                                    });
                                }

                                parent.spawn_bundle(SpriteBundle {
                                    transform: Transform {
                                        translation: Vec3::new(
                                            match side {
                                                TileSide::Left => -40.,
                                                TileSide::Right => 40.,
                                            },
                                            0.,
                                            10.,
                                        ),

                                        ..Default::default()
                                    },
                                    sprite: Sprite {
                                        custom_size: Some(Vec2::new(card_size, card_size)),
                                        flip_y: match direction {
                                            CycleDirection::Up => false,
                                            CycleDirection::Down => true,
                                        },
                                        ..Default::default()
                                    },
                                    texture: asset_server.load("cycle_arrow.png"),
                                    ..Default::default()
                                });

                                format!(
                                    "Cycle {} x {} - {}",
                                    match direction {
                                        CycleDirection::Up => "up",
                                        CycleDirection::Down => "down",
                                    },
                                    times,
                                    match side {
                                        TileSide::Left => "left",
                                        TileSide::Right => "right",
                                    }
                                )
                            }
                        };

                        parent.spawn_bundle(Text2dBundle {
                            text: Text::with_section(
                                card_as_text,
                                TextStyle {
                                    font: asset_server.load("ReadexPro-Regular.ttf"),
                                    font_size: 20.,
                                    color: Color::FUCHSIA,
                                },
                                TextAlignment {
                                    vertical: VerticalAlign::Center,
                                    horizontal: HorizontalAlign::Center,
                                },
                            ),
                            text_2d_size: Text2dSize {
                                size: Size {
                                    width: 200.,
                                    ..Default::default()
                                },
                            },
                            transform: Transform {
                                translation: Vec3::new(-10., 0., 20.),
                                rotation: Quat::from_rotation_z(PI / 4.),
                                ..Default::default()
                            },
                            ..Default::default()
                        });
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

        event_update_cards_style.send(UpdateCardsStyle);
    }
}

fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut match_state: ResMut<MatchState>,
    mut update_tiles_position_event: EventWriter<UpdateTilesPosition>,
    mut event_update_cards_style: EventWriter<UpdateCardsStyle>,
) {
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
                event_update_cards_style.send(UpdateCardsStyle);
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
                event_update_cards_style.send(UpdateCardsStyle);
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

                        // Update cards position.
                        update_tiles_position_event.send(UpdateTilesPosition);

                        // Check for victory.
                        let natures_in_the_columns_match = match_state
                            .left_col
                            .iter()
                            .zip(match_state.right_col.iter())
                            .all(|(l, r)| l.nature == r.nature);
                        if natures_in_the_columns_match {
                            info!("Victory");
                        } else {
                            info!("Retry");
                        }
                    }

                    event_update_cards_style.send(UpdateCardsStyle);
                }
            }
            _ => (),
        }
    }
}

#[derive(Component)]
struct Cursor;

const CURSOR_Y_POS: f32 = -510.;

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

struct UpdateTilesPosition;

fn update_tiles_position(
    mut update_tiles_position_event: EventReader<UpdateTilesPosition>,
    match_state: Res<MatchState>,
    mut q: Query<(Entity, &mut Transform), With<Tile>>,
) {
    for _ in update_tiles_position_event.iter() {
        match match_state.as_ref() {
            MatchState::Playing(match_state) => {
                let (tiles_pos_left, tiles_pos_right) =
                    tiles_layout_poss(TILE_POS_Y_GAP, match_state.left_col.len());

                for (entity, mut transform) in q.iter_mut() {
                    let (i, _tile_data, side) = match_state
                        .left_col
                        .iter()
                        .zip(match_state.right_col.iter())
                        .enumerate()
                        .find_map(|(i, (l, r))| {
                            if l.id == entity {
                                Some((i, l, TileSide::Left))
                            } else if r.id == entity {
                                Some((i, r, TileSide::Right))
                            } else {
                                None
                            }
                        })
                        .unwrap();
                    let pos = match side {
                        TileSide::Left => tiles_pos_left[i],
                        TileSide::Right => tiles_pos_right[i],
                    };
                    transform.translation = Vec3::new(pos.x, pos.y, 0.);
                }
            }
            _ => unreachable!(),
        }
    }
}

struct UpdateCardsStyle;

fn update_cards_style(
    mut update_cards_position_event: EventReader<UpdateCardsStyle>,
    match_state: Res<MatchState>,
    mut q: Query<(Entity, &mut Transform), With<Card>>,
) {
    for _ in update_cards_position_event.iter() {
        match match_state.as_ref() {
            MatchState::Playing(match_state) => {
                for (entity, mut transform) in q.iter_mut() {
                    let is_hovered = match match_state.hovered_card {
                        Some(i) => match_state.cards[i].id == entity,
                        None => false,
                    };
                    let is_used = match_state
                        .cards
                        .iter()
                        .find_map(|c| {
                            if c.id == entity {
                                Some(match c.used {
                                    Some(_) => true,
                                    None => false,
                                })
                            } else {
                                None
                            }
                        })
                        .unwrap();

                    let scale = if is_used {
                        0.7
                    } else if is_hovered {
                        1.1
                    } else {
                        1.
                    };
                    transform.scale = Vec3::new(scale, scale, scale);
                }
            }
            _ => unreachable!(),
        }
    }
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_event::<StartMatchEvent>()
        .add_event::<UpdateTilesPosition>()
        .add_event::<UpdateCardsStyle>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_startup_system(setup_cursor)
        .add_system(start_match)
        .add_system(handle_input)
        .add_system(update_cursor)
        .add_system(update_tiles_position)
        .add_system(update_cards_style)
        .run();
}
