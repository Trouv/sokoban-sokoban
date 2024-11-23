use crate::{sokoban::SokobanBlock, utils::any_match_filter, AssetHolder, GameState, UNIT_LENGTH};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub struct OutOfBoundsPlugin;

impl Plugin for OutOfBoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            spawn_level_boundaries.run_if(any_match_filter::<Added<LevelIid>>.and_then(
                in_state(GameState::Graveyard).or_else(in_state(GameState::LevelTransition)),
            )),
        );
    }
}

fn spawn_level_boundaries(
    mut commands: Commands,
    level_iids: Query<(Entity, &LevelIid), Added<LevelIid>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    asset_holder: Res<AssetHolder>,
) {
    let ldtk_project = ldtk_project_assets
        .get(&asset_holder.ldtk)
        .expect("ldtk project should be loaded if LevelIids are spawned");
    level_iids.for_each(|(level_entity, level_iid)| {
        let level = ldtk_project
            .get_raw_level_by_iid(level_iid.get())
            .expect("spawned level should exist in ldtk project");

        let level_grid_size = IVec2::new(level.px_wid, level.px_hei) / UNIT_LENGTH;

        commands
            .entity(level_entity)
            .with_children(|child_commands| {
                for x in -1..=level_grid_size.x {
                    // spawn bottom wall
                    child_commands
                        .spawn(SokobanBlock::Static)
                        .insert(GridCoords::new(x, -1));

                    // spawn top wall
                    child_commands
                        .spawn(SokobanBlock::Static)
                        .insert(GridCoords::new(x, level_grid_size.y));
                }

                for y in 0..level_grid_size.y {
                    // spawn left wall
                    child_commands
                        .spawn(SokobanBlock::Static)
                        .insert(GridCoords::new(-1, y));

                    // spawn right wall
                    child_commands
                        .spawn(SokobanBlock::Static)
                        .insert(GridCoords::new(level_grid_size.x, y));
                }
            });
    })
}
