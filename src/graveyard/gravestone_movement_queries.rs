use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_ldtk::prelude::*;

use crate::graveyard::{arrow_block::MovementTile, gravestone::GraveId};

use super::volatile::Volatile;

#[derive(Debug, SystemParam)]
pub struct GravestoneMovementQueries<'w, 's> {
    gravestones: Query<'w, 's, (&'static GridCoords, &'static GraveId, &'static Volatile)>,
    movement_tiles: Query<'w, 's, (&'static GridCoords, &'static MovementTile)>,
}

impl<'w, 's> GravestoneMovementQueries<'w, 's> {
    pub fn find_movement(&self, grave_id: &GraveId) -> Option<&MovementTile> {
        self.gravestones
            .iter()
            .find(|(_, this_grave_id, volatile)| {
                **volatile == Volatile::Solid && &grave_id == this_grave_id
            })
            .and_then(|(grid_coords, ..)| {
                self.movement_tiles
                    .iter()
                    .find(|(this_grid_coords, _)| &grid_coords == this_grid_coords)
            })
            .map(|(_, movement_tile)| movement_tile)
    }
}
