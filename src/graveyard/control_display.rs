//! Plugin providing functionality for the graveyard UI element showing the current controls.
use crate::{
    camera::PlayZonePortion,
    graveyard::gravestone::GraveId,
    ui::{
        action::UiAction,
        icon_button::{IconButton, IconButtonBundle, IconButtonSet},
    },
    ui_atlas_image::UiAtlasImage,
    utils::any_match_filter,
    GameState,
};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use super::{
    arrow_block::MovementTile, gravestone_movement_queries::GravestoneMovementQueries,
    volatile::Sublimation, GraveyardAction,
};

/// Plugin providing functionality for the graveyard UI element showing the current controls.
pub struct ControlDisplayPlugin;

impl Plugin for ControlDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LevelTransition), spawn_control_display)
            .add_systems(
                Update,
                update_grave_action_buttons
                    .run_if(in_state(GameState::Graveyard).and_then(
                        any_match_filter::<(
                            Changed<GridCoords>,
                            Or<(With<MovementTile>, With<GraveId>)>,
                        )>,
                    ))
                    .after(Sublimation)
                    .before(IconButtonSet),
            );
    }
}

/// Component that marks the main ControlDisplay UI Node.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct ControlDisplay;

/// Asset collection for loading/storing assets relevant to the control display.
#[derive(Clone, Debug, AssetCollection, Resource)]
pub struct ControlDisplayAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 9, rows = 9))]
    #[asset(path = "textures/movement-table-actions.png")]
    movement_table_actions: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 3, rows = 1))]
    #[asset(path = "textures/graveyard-actions.png")]
    graveyard_actions: Handle<TextureAtlas>,
}

fn spawn_control_display(
    mut commands: Commands,
    play_zone_portion: Res<PlayZonePortion>,
    mut already_spawned: Local<bool>,
    assets: Res<ControlDisplayAssets>,
) {
    if !*already_spawned {
        let control_zone_ratio = 1. - **play_zone_portion;

        commands
            .spawn(NodeBundle {
                background_color: BackgroundColor(Color::NONE),
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::SpaceAround,
                    align_content: AlignContent::Center,
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100. * control_zone_ratio),
                    height: Val::Percent(100.),
                    top: Val::Percent(0.),
                    right: Val::Percent(0.),
                    ..Default::default()
                },
                z_index: ZIndex::Local(-1),
                ..Default::default()
            })
            .insert(ControlDisplay)
            .with_children(|control_display| {
                // spawn grave ids
                control_display
                    .spawn(NodeBundle {
                        style: Style {
                            aspect_ratio: Some(3. / 2.),
                            width: Val::Percent(80.),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|movement_table_action_container| {
                        // spawn northwest
                        movement_table_action_container
                            .spawn(IconButtonBundle::new_with_absolute_position(
                                IconButton::NoIcon,
                                UiRect {
                                    top: Val::Percent(0.),
                                    left: Val::Percent(0.),
                                    bottom: Val::Percent(50.),
                                    right: Val::Percent(200. / 3.),
                                },
                            ))
                            .insert(UiAction(GraveId::Northwest));

                        // spawn north
                        movement_table_action_container
                            .spawn(IconButtonBundle::new_with_absolute_position(
                                IconButton::NoIcon,
                                UiRect {
                                    top: Val::Percent(0.),
                                    left: Val::Percent(100. / 3.),
                                    bottom: Val::Percent(50.),
                                    right: Val::Percent(100. / 3.),
                                },
                            ))
                            .insert(UiAction(GraveId::North));

                        // spawn northeast
                        movement_table_action_container
                            .spawn(IconButtonBundle::new_with_absolute_position(
                                IconButton::NoIcon,
                                UiRect {
                                    top: Val::Percent(0.),
                                    left: Val::Percent(200. / 3.),
                                    bottom: Val::Percent(50.),
                                    right: Val::Percent(0.),
                                },
                            ))
                            .insert(UiAction(GraveId::Northeast));

                        // spawn west
                        movement_table_action_container
                            .spawn(IconButtonBundle::new_with_absolute_position(
                                IconButton::NoIcon,
                                UiRect {
                                    top: Val::Percent(50.),
                                    left: Val::Percent(0.),
                                    bottom: Val::Percent(0.),
                                    right: Val::Percent(200. / 3.),
                                },
                            ))
                            .insert(UiAction(GraveId::West));

                        // spawn south
                        movement_table_action_container
                            .spawn(IconButtonBundle::new_with_absolute_position(
                                IconButton::NoIcon,
                                UiRect {
                                    top: Val::Percent(50.),
                                    left: Val::Percent(100. / 3.),
                                    bottom: Val::Percent(0.),
                                    right: Val::Percent(100. / 3.),
                                },
                            ))
                            .insert(UiAction(GraveId::South));

                        // spawn east
                        movement_table_action_container
                            .spawn(IconButtonBundle::new_with_absolute_position(
                                IconButton::NoIcon,
                                UiRect {
                                    top: Val::Percent(50.),
                                    left: Val::Percent(200. / 3.),
                                    bottom: Val::Percent(0.),
                                    right: Val::Percent(0.),
                                },
                            ))
                            .insert(UiAction(GraveId::East));
                    });

                // spawn other grave actions
                control_display
                    .spawn(NodeBundle {
                        style: Style {
                            aspect_ratio: Some(3.),
                            width: Val::Percent(80.),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|graveyard_action_container| {
                        // spawn undo
                        graveyard_action_container
                            .spawn(IconButtonBundle::new(IconButton::AtlasImageIcon(
                                UiAtlasImage {
                                    texture_atlas: assets.graveyard_actions.clone(),
                                    index: 0,
                                },
                            )))
                            .insert(UiAction(GraveyardAction::Undo));

                        // spawn restart
                        graveyard_action_container
                            .spawn(IconButtonBundle::new(IconButton::AtlasImageIcon(
                                UiAtlasImage {
                                    texture_atlas: assets.graveyard_actions.clone(),
                                    index: 1,
                                },
                            )))
                            .insert(UiAction(GraveyardAction::Restart));

                        // spawn pause
                        graveyard_action_container
                            .spawn(IconButtonBundle::new(IconButton::AtlasImageIcon(
                                UiAtlasImage {
                                    texture_atlas: assets.graveyard_actions.clone(),
                                    index: 2,
                                },
                            )))
                            .insert(UiAction(GraveyardAction::Pause));
                    });
            });

        *already_spawned = true;
    }
}

fn update_grave_action_buttons(
    gravestone_movement_queries: GravestoneMovementQueries,
    mut grave_action_buttons: Query<(&mut IconButton, &UiAction<GraveId>)>,
    assets: Res<ControlDisplayAssets>,
) {
    for (mut icon_button, action) in &mut grave_action_buttons {
        *icon_button = match gravestone_movement_queries.find_movement(&action) {
            Some(movement_tile) => {
                let index = movement_tile.tileset_index();
                IconButton::AtlasImageIcon(UiAtlasImage {
                    texture_atlas: assets.movement_table_actions.clone(),
                    index,
                })
            }
            None => IconButton::NoIcon,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sokoban::Direction;

    use super::*;
    use bevy::reflect::Enum;
    use rand::prelude::*;

    fn app_setup() -> App {
        let mut app = App::new();

        app.add_state::<GameState>()
            .add_plugins(ControlDisplayPlugin)
            .insert_resource(PlayZonePortion(0.5));

        app
    }

    fn asset_setup(app: &mut App) -> ControlDisplayAssets {
        let mut rng = rand::thread_rng();
        let control_display_assets = ControlDisplayAssets {
            movement_table_actions: Handle::weak_from_u128(rng.gen()),
            graveyard_actions: Handle::weak_from_u128(rng.gen()),
        };

        app.insert_resource(control_display_assets.clone());

        control_display_assets
    }

    struct GravestoneMovementTilePairTestSpawner {
        northwest_grave_tile: MovementTile,
        north_grave_tile: MovementTile,
        northeast_grave_tile: MovementTile,
        west_grave_tile: MovementTile,
        south_grave_tile: MovementTile,
        east_grave_tile: MovementTile,
    }

    struct GravestoneMovementTilePair {
        gravestone: Entity,
        movement_tile: Entity,
    }

    struct SpawnedGravestoneMovementTilePairs {
        northwest_pair: GravestoneMovementTilePair,
        north_pair: GravestoneMovementTilePair,
        northeast_pair: GravestoneMovementTilePair,
        west_pair: GravestoneMovementTilePair,
        south_pair: GravestoneMovementTilePair,
        east_pair: GravestoneMovementTilePair,
    }

    fn hash_movement_to_grid_coords(movement_tile: &MovementTile) -> GridCoords {
        GridCoords::new(
            9 - movement_tile.row_move().variant_index() as i32,
            movement_tile.column_move().variant_index() as i32,
        )
    }

    impl GravestoneMovementTilePairTestSpawner {
        fn new_valid() -> Self {
            GravestoneMovementTilePairTestSpawner {
                northwest_grave_tile: MovementTile::new(Direction::Up, Direction::Left),
                north_grave_tile: MovementTile::new(Direction::Up, Direction::Up),
                northeast_grave_tile: MovementTile::new(Direction::Up, Direction::Right),
                west_grave_tile: MovementTile::new(Direction::Left, Direction::Left),
                south_grave_tile: MovementTile::new(Direction::Down, Direction::Down),
                east_grave_tile: MovementTile::new(Direction::Right, Direction::Right),
            }
        }

        fn spawn(self, world: &mut World) -> SpawnedGravestoneMovementTilePairs {
            SpawnedGravestoneMovementTilePairs {
                northwest_pair: GravestoneMovementTilePair {
                    gravestone: world
                        .spawn(hash_movement_to_grid_coords(&self.northwest_grave_tile))
                        .insert(GraveId::Northwest)
                        .id(),
                    movement_tile: world
                        .spawn(hash_movement_to_grid_coords(&self.northwest_grave_tile))
                        .insert(self.northwest_grave_tile)
                        .id(),
                },
                north_pair: GravestoneMovementTilePair {
                    gravestone: world
                        .spawn(hash_movement_to_grid_coords(&self.north_grave_tile))
                        .insert(GraveId::North)
                        .id(),
                    movement_tile: world
                        .spawn(hash_movement_to_grid_coords(&self.north_grave_tile))
                        .insert(self.north_grave_tile)
                        .id(),
                },
                northeast_pair: GravestoneMovementTilePair {
                    gravestone: world
                        .spawn(hash_movement_to_grid_coords(&self.northeast_grave_tile))
                        .insert(GraveId::Northeast)
                        .id(),
                    movement_tile: world
                        .spawn(hash_movement_to_grid_coords(&self.northeast_grave_tile))
                        .insert(self.northeast_grave_tile)
                        .id(),
                },
                west_pair: GravestoneMovementTilePair {
                    gravestone: world
                        .spawn(hash_movement_to_grid_coords(&self.west_grave_tile))
                        .insert(GraveId::West)
                        .id(),
                    movement_tile: world
                        .spawn(hash_movement_to_grid_coords(&self.west_grave_tile))
                        .insert(self.west_grave_tile)
                        .id(),
                },
                south_pair: GravestoneMovementTilePair {
                    gravestone: world
                        .spawn(hash_movement_to_grid_coords(&self.south_grave_tile))
                        .insert(GraveId::South)
                        .id(),
                    movement_tile: world
                        .spawn(hash_movement_to_grid_coords(&self.south_grave_tile))
                        .insert(self.south_grave_tile)
                        .id(),
                },
                east_pair: GravestoneMovementTilePair {
                    gravestone: world
                        .spawn(hash_movement_to_grid_coords(&self.east_grave_tile))
                        .insert(GraveId::East)
                        .id(),
                    movement_tile: world
                        .spawn(hash_movement_to_grid_coords(&self.east_grave_tile))
                        .insert(self.east_grave_tile)
                        .id(),
                },
            }
        }
    }

    fn initial_state_changes(app: &mut App) {
        app.update();

        app.world
            .insert_resource(NextState(Some(GameState::LevelTransition)));

        app.update();

        app.world
            .insert_resource(NextState(Some(GameState::Graveyard)));

        app.update();
    }

    fn get_icon_button_for_action<A: Clone + Send + Sync + PartialEq + 'static>(
        app: &mut App,
        action: A,
    ) -> &IconButton {
        app.world
            .query::<(&IconButton, &UiAction<A>)>()
            .iter(&app.world)
            .find(|(_, a)| ***a == action)
            .map(|(i, _)| i)
            .unwrap()
    }

    #[test]
    fn plugin_spawns_all_buttons() {
        let mut app = app_setup();
        let assets = asset_setup(&mut app);
        GravestoneMovementTilePairTestSpawner::new_valid().spawn(&mut app.world);
        initial_state_changes(&mut app);

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::Northwest),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 22
            })
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::North),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 20
            })
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::Northeast),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 26
            })
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::West),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 40
            }),
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::South),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 60
            }),
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::East),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 80
            }),
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveyardAction::Undo),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.graveyard_actions.clone(),
                index: 0
            }),
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveyardAction::Restart),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.graveyard_actions.clone(),
                index: 1
            }),
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveyardAction::Pause),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.graveyard_actions.clone(),
                index: 2
            }),
        );
    }

    #[test]
    fn grave_id_buttons_change_according_to_movement_table() {
        let mut app = app_setup();
        let assets = asset_setup(&mut app);
        let spawned_gravestone_movement_tile_pairs =
            GravestoneMovementTilePairTestSpawner::new_valid().spawn(&mut app.world);
        initial_state_changes(&mut app);

        // check initial values of a couple buttons
        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::North),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 20
            })
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::West),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 40
            }),
        );

        // change the movement table and check those buttons again
        let mut north_grave = app
            .world
            .entity_mut(spawned_gravestone_movement_tile_pairs.north_pair.gravestone);

        let target_movement_tile = MovementTile::new(Direction::Up, Direction::Down);

        *north_grave.get_mut::<GridCoords>().unwrap() =
            hash_movement_to_grid_coords(&target_movement_tile);

        app.world
            .spawn(hash_movement_to_grid_coords(&target_movement_tile))
            .insert(target_movement_tile);

        app.world
            .despawn(spawned_gravestone_movement_tile_pairs.west_pair.gravestone);

        app.update();

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::North),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 24
            })
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::West),
            &IconButton::NoIcon,
        );
    }
}
