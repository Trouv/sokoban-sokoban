//! Plugin providing functionality for sokoban-style movement and collision to LDtk levels.
//!
//! You should use `bevy_ecs_ldtk` to load the levels.
//! Spawn entities with `GridCoords` (from `bevy_ecs_ldtk`) and [SokobanBlock]s to give them
//! sokoban-style collision.
//! Then, move entities around with the [SokobanCommands] system parameter.
use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_easings::*;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};
use std::{
    hash::Hash,
    marker::PhantomData,
    ops::{Add, AddAssign},
};
use thiserror::Error;

/// Sets used by sokoban systems
#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum SokobanSets {
    /// Set for the system that updates the visual position of sokoban entities via bevy_easings.
    EaseMovement,
    /// Set for the system that updates the logical position of sokoban entities.
    LogicalMovement,
}

/// Plugin providing functionality for sokoban-style movement and collision to LDtk levels.
pub struct SokobanPlugin<S, P, D>
where
    S: States,
    P: Push + Component,
    for<'d> IVec2: Add<&'d D, Output = IVec2>,
    D: Hash + PartialEq + Eq + Clone + Send + Sync + 'static,
{
    state: S,
    layer_identifier: SokobanLayerIdentifier,
    phantom_push_component: PhantomData<P>,
    phantom_direction: PhantomData<D>,
}

impl<S, P, D> SokobanPlugin<S, P, D>
where
    S: States,
    P: Push + Component,
    for<'d> IVec2: Add<&'d D, Output = IVec2>,
    D: Hash + PartialEq + Eq + Clone + Send + Sync + 'static,
{
    /// Constructor for the plugin.
    ///
    /// Allows the user to specify a particular iyes_loopless state to run the plugin in.
    ///
    /// The `layer_identifier` should refer to a non-entity layer in LDtk that can be treated as
    /// the Sokoban grid.
    /// This layer should have the tile-size and dimensions for your desired sokoban functionality.
    pub fn new(state: S, layer_identifier: impl Into<String>) -> Self {
        let layer_identifier = SokobanLayerIdentifier(layer_identifier.into());
        SokobanPlugin {
            state,
            layer_identifier,
            phantom_push_component: PhantomData,
            phantom_direction: PhantomData,
        }
    }
}

impl<S, P, D> Plugin for SokobanPlugin<S, P, D>
where
    S: States,
    P: Push + Component,
    for<'d> IVec2: Add<&'d D, Output = IVec2>,
    D: Hash + PartialEq + Eq + Clone + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_event::<SokobanCommand<Direction>>()
            .add_event::<PushEvent<Direction>>()
            .insert_resource(self.layer_identifier.clone())
            .add_systems(
                Update,
                flush_sokoban_commands::<P, Direction>
                    .run_if(in_state(self.state.clone()))
                    .run_if(on_event::<SokobanCommand<Direction>>())
                    .in_set(SokobanSets::LogicalMovement),
            )
            // Systems with potential easing end/beginning collisions cannot be in CoreSet::Update
            // see https://github.com/vleue/bevy_easings/issues/23
            .add_systems(
                PostUpdate,
                ease_movement
                    .run_if(in_state(self.state.clone()))
                    .in_set(SokobanSets::EaseMovement),
            );
    }
}

/// Resource referring to the LDtk layer that should be treated as a sokoban grid.
#[derive(Debug, Clone, Deref, DerefMut, Resource)]
struct SokobanLayerIdentifier(String);

/// Enumerates the four directions that sokoban blocks can be pushed in.
#[derive(Copy, Clone, Default, Eq, PartialEq, Debug, Hash, Reflect)]
pub enum Direction {
    #[default]
    Zero,
    UpRight,
    /// North direction.
    Up,
    UpLeft,
    /// West direction.
    Left,
    DownLeft,
    /// South direction.
    Down,
    DownRight,
    /// East direction.
    Right,
}

impl Add<&Direction> for IVec2 {
    type Output = IVec2;

    fn add(self, direction: &Direction) -> IVec2 {
        match direction {
            Direction::Zero => self + IVec2::ZERO,
            Direction::UpRight => self + IVec2::new(1, 1),
            Direction::Up => self + IVec2::Y,
            Direction::UpLeft => self + IVec2::new(-1, 1),
            Direction::Left => self - IVec2::X,
            Direction::DownLeft => self + IVec2::new(-1, -1),
            Direction::Down => self - IVec2::Y,
            Direction::DownRight => self + IVec2::new(1, -1),
            Direction::Right => self + IVec2::X,
        }
    }
}

#[derive(Debug, Error)]
#[error("Directions must have coordinates of 0s and 1s")]
pub struct OutOfBoundsDirection;

impl TryFrom<&IVec2> for Direction {
    type Error = OutOfBoundsDirection;

    fn try_from(value: &IVec2) -> Result<Direction, OutOfBoundsDirection> {
        match (value.x, value.y) {
            (0, 0) => Ok(Direction::Zero),
            (1, 1) => Ok(Direction::UpRight),
            (0, 1) => Ok(Direction::Up),
            (-1, 1) => Ok(Direction::UpLeft),
            (-1, 0) => Ok(Direction::Left),
            (-1, -1) => Ok(Direction::DownLeft),
            (0, -1) => Ok(Direction::Down),
            (1, -1) => Ok(Direction::DownRight),
            (1, 0) => Ok(Direction::Right),
            _ => Err(OutOfBoundsDirection),
        }
    }
}

impl Direction {
    fn try_add(self, rhs: Direction) -> Result<Direction, OutOfBoundsDirection> {
        Direction::try_from(&(IVec2::ZERO + &self + &rhs))
    }
}

impl Add<Direction> for Direction {
    type Output = Direction;

    fn add(self, rhs: Direction) -> Self::Output {
        self.try_add(rhs).unwrap()
    }
}

impl AddAssign for Direction {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

/// Enumerates commands that can be performed via [SokobanCommands].
#[derive(Debug, Clone, Event)]
pub enum SokobanCommand<D>
where
    for<'d> IVec2: Add<&'d D, Output = IVec2>,
    D: Hash + PartialEq + Eq + Clone + Send + Sync + 'static,
{
    /// Move a [SokobanBlock] entity in the given direction.
    Move {
        /// The [SokobanBlock] entity to move.
        entity: Entity,
        /// The direction to move the block in.
        direction: D,
    },
}

/// System parameter providing an interface for commanding the SokobanPlugin.
#[derive(SystemParam)]
pub struct SokobanCommands<'w, D>
where
    for<'d> IVec2: Add<&'d D, Output = IVec2>,
    D: Hash + PartialEq + Eq + Clone + Send + Sync + 'static,
{
    writer: EventWriter<'w, SokobanCommand<D>>,
}

impl<'w, D> SokobanCommands<'w, D>
where
    for<'d> IVec2: Add<&'d D, Output = IVec2>,
    D: Hash + PartialEq + Eq + Clone + Send + Sync + 'static,
{
    /// Move a [SokobanBlock] entity in the given direction.
    ///
    /// Will perform the necessary collision checks and block pushes.
    pub fn move_block(&mut self, entity: Entity, direction: D) {
        self.writer.send(SokobanCommand::Move { entity, direction });
    }
}

/// Component defining the behavior of sokoban entities on collision.
#[derive(Copy, Clone, Default, Eq, PartialEq, Debug, Hash, Component)]
pub enum SokobanBlock {
    #[default]
    /// The entity cannot move, push, or be pushed - but can block movement.
    Static,
    /// The entity can move, push, or be pushed.
    Dynamic,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum PusherResult {
    #[default]
    NotBlocked,
    Blocked,
}

impl PusherResult {
    fn reduce(&self, other: &PusherResult) -> PusherResult {
        match (self, other) {
            (PusherResult::NotBlocked, PusherResult::NotBlocked) => PusherResult::NotBlocked,
            _ => PusherResult::Blocked,
        }
    }
}

pub enum PusheeResult {
    NotPushed,
    Pushed,
}

impl PusheeResult {
    fn reduce(&self, other: &PusheeResult) -> PusheeResult {
        match (self, other) {
            (PusheeResult::NotPushed, PusheeResult::NotPushed) => PusheeResult::NotPushed,
            _ => PusheeResult::Pushed,
        }
    }
}

pub trait Push {
    fn push(&self, pushee: &Self) -> (PusherResult, PusheeResult);
}

impl Push for SokobanBlock {
    fn push(&self, pushee: &Self) -> (PusherResult, PusheeResult) {
        match (self, pushee) {
            (_, SokobanBlock::Static) => (PusherResult::Blocked, PusheeResult::NotPushed),
            (SokobanBlock::Static, _) => (PusherResult::Blocked, PusheeResult::Pushed),
            (SokobanBlock::Dynamic, _) => (PusherResult::NotBlocked, PusheeResult::Pushed),
        }
    }
}

impl SokobanBlock {
    /// Constructor returning [SokobanBlock::Static].
    ///
    /// Compatible with the `with` attribute for `#[derive(LdtkEntity)]`:
    /// ```
    /// use bevy_ecs_ldtk::*;
    ///
    /// #[derive(Bundle, LdtkEntity)]
    /// struct MyLdtkEntity {
    ///     #[grid_coords]
    ///     grid_coords: GridCoords,
    ///     #[with(SokobanBlock::new_static)]
    ///     sokoban_block: SokobanBlock,
    /// }
    /// ```
    pub fn new_static(_: &EntityInstance) -> SokobanBlock {
        SokobanBlock::Static
    }

    /// Constructor returning [SokobanBlock::Dynamic].
    ///
    /// Compatible with the `with` attribute for `#[derive(LdtkEntity)]`:
    /// ```
    /// use bevy_ecs_ldtk::*;
    ///
    /// #[derive(Bundle, LdtkEntity)]
    /// struct MyLdtkEntity {
    ///     #[grid_coords]
    ///     grid_coords: GridCoords,
    ///     #[with(SokobanBlock::new_dynamic)]
    ///     sokoban_block: SokobanBlock,
    /// }
    /// ```
    pub fn new_dynamic(_: &EntityInstance) -> SokobanBlock {
        SokobanBlock::Dynamic
    }
}

/// Component that marks [SokobanBlock]s that should fire [PushEvent]s when they push other blocks.
#[derive(Clone, Default, Debug, Component)]
pub struct PushTracker;

/// Event that fires when a [PushTracker] entity pushes other [SokobanBlock]s.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Event)]
pub struct PushEvent<D>
where
    for<'d> IVec2: Add<&'d D, Output = IVec2>,
    D: Hash + PartialEq + Eq + Clone + Send + Sync + 'static,
{
    /// The [PushTracker] entity that pushed other [SokobanBlock]s.
    pub pusher: Entity,
    /// The direction of the push.
    pub direction: D,
}

fn ease_movement(
    mut commands: Commands,
    mut grid_coords_query: Query<
        (Entity, &GridCoords, &Transform),
        (Changed<GridCoords>, With<SokobanBlock>),
    >,
    layers: Query<&LayerMetadata>,
    layer_id: Res<SokobanLayerIdentifier>,
) {
    for (entity, &grid_coords, transform) in grid_coords_query.iter_mut() {
        if let Some(LayerMetadata { grid_size, .. }) =
            layers.iter().find(|l| l.identifier == **layer_id)
        {
            let xy = grid_coords_to_translation(grid_coords, IVec2::splat(*grid_size));

            commands.entity(entity).insert(transform.ease_to(
                Transform::from_xyz(xy.x, xy.y, transform.translation.z),
                EaseFunction::CubicOut,
                EasingType::Once {
                    duration: std::time::Duration::from_millis(110),
                },
            ));
        }
    }
}

#[derive(Clone, Default, Debug)]
struct EntityCollisionGeographicMap<'a, P>
where
    P: Push + Component,
{
    coordinate_table: HashMap<IVec2, HashSet<Entity>>,
    entity_table: HashMap<Entity, (IVec2, &'a P)>,
}

impl<'a, P> FromIterator<(Entity, IVec2, &'a P)> for EntityCollisionGeographicMap<'a, P>
where
    P: Push + Component,
{
    fn from_iter<T: IntoIterator<Item = (Entity, IVec2, &'a P)>>(iter: T) -> Self {
        iter.into_iter().fold(
            Self::new(),
            |EntityCollisionGeographicMap {
                 mut coordinate_table,
                 mut entity_table,
             },
             (entity, coordinate, push_block)| {
                coordinate_table
                    .entry(coordinate)
                    .or_default()
                    .insert(entity);
                entity_table.insert(entity, (coordinate, push_block));

                EntityCollisionGeographicMap {
                    coordinate_table,
                    entity_table,
                }
            },
        )
    }
}

impl<'a, P> EntityCollisionGeographicMap<'a, P>
where
    P: Push + Component,
{
    fn new() -> Self {
        EntityCollisionGeographicMap {
            coordinate_table: HashMap::new(),
            entity_table: HashMap::new(),
        }
    }

    fn get_coordinate_and_block(&self, entity: &Entity) -> Option<&(IVec2, &'a P)> {
        self.entity_table.get(entity)
    }

    fn get_coordinate(&self, entity: &Entity) -> Option<&IVec2> {
        self.get_coordinate_and_block(entity)
            .map(|(coordinate, _)| coordinate)
    }

    fn get_block(&self, entity: &Entity) -> Option<&'a P> {
        self.get_coordinate_and_block(entity)
            .map(|(_, block)| *block)
    }

    fn get_entities_at_coords(&self, coordinate: &IVec2) -> Option<&HashSet<Entity>> {
        self.coordinate_table.get(coordinate)
    }

    /// returns a list of entities that would be pushed
    fn simulate_move_entity<D>(
        &self,
        pusher_entity: &Entity,
        direction: &D,
    ) -> (PusherResult, HashSet<Entity>, HashSet<PushEvent<D>>)
    where
        for<'d> IVec2: Add<&'d D, Output = IVec2>,
        D: Hash + PartialEq + Eq + Clone + Send + Sync + 'static,
    {
        let Some((pusher_coordinate, pusher_block)) = self.get_coordinate_and_block(pusher_entity)
        else {
            return default();
        };

        let destination = *pusher_coordinate + direction;
        if &destination == pusher_coordinate {
            return default();
        }

        let (pusher_result, mut moved_entities, mut push_events) = self
            .get_entities_at_coords(&destination)
            .iter()
            .copied()
            .flatten()
            .map(|pushee_entity| {
                let pushee_block = self
                    .get_block(pushee_entity)
                    .expect("entities in coordinate table should also exist in entity table");

                let (our_pusher_result, pushee_result) = pusher_block.push(pushee_block);

                let (their_pusher_result, moved_entities, push_events) = match pushee_result {
                    PusheeResult::Pushed => self.simulate_move_entity(pushee_entity, direction),
                    PusheeResult::NotPushed => default(),
                };

                let pusher_result = our_pusher_result.reduce(&their_pusher_result);

                (pusher_result, moved_entities, push_events)
            })
            .reduce(
                |(
                    previous_pusher_result,
                    mut previous_moved_entities,
                    mut previous_push_events,
                ),
                 (pusher_result, moved_entities, push_events)| {
                    previous_moved_entities.extend(moved_entities);
                    previous_push_events.extend(push_events);

                    (
                        previous_pusher_result.reduce(&pusher_result),
                        previous_moved_entities,
                        previous_push_events,
                    )
                },
            )
            .unwrap_or_default();

        if !moved_entities.is_empty() {
            push_events.insert(PushEvent {
                pusher: *pusher_entity,
                direction: direction.clone(),
            });
        }

        if pusher_result == PusherResult::NotBlocked {
            moved_entities.insert(*pusher_entity);
        }

        (pusher_result, moved_entities, push_events)
    }
}

fn flush_sokoban_commands<P, D>(
    mut grid_coords_query: Query<(Entity, &mut GridCoords, &P, Has<PushTracker>)>,
    mut sokoban_commands: EventReader<SokobanCommand<D>>,
    mut push_events: EventWriter<PushEvent<D>>,
) where
    P: Push + Component,
    for<'d> IVec2: Add<&'d D, Output = IVec2>,
    D: Hash + PartialEq + Eq + Clone + Send + Sync + 'static,
{
    for sokoban_command in sokoban_commands.read() {
        let SokobanCommand::Move { entity, direction } = sokoban_command;

        let (_, entities_to_move, push_events_to_send) = {
            // regenerate map per command to get map updates from previous command
            let entity_collision_geographic_map = grid_coords_query
                .iter()
                .map(|(entity, grid_coords, sokoban_block, _)| {
                    (entity, IVec2::from(*grid_coords), sokoban_block)
                })
                .collect::<EntityCollisionGeographicMap<P>>();

            entity_collision_geographic_map.simulate_move_entity(entity, direction)
        };

        entities_to_move.iter().for_each(|entity_to_move| {
            let mut grid_coords = grid_coords_query
                .get_component_mut::<GridCoords>(*entity_to_move)
                .expect("pushed entity should be valid sokoban entity");

            let new_coords = IVec2::from(*grid_coords) + direction;
            *grid_coords = GridCoords::from(new_coords);
        });

        push_events_to_send
            .into_iter()
            .filter(|push_event| {
                let (.., is_push_tracker) = grid_coords_query
                    .get(push_event.pusher)
                    .expect("entity source from query should still exist in query");

                is_push_tracker
            })
            .for_each(|push_event| push_events.send(push_event));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;

    #[test]
    fn push_dynamic_into_empty() {
        let pusher = Entity::from_raw(0);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[1][1] = Some((pusher, SokobanBlock::Dynamic));

        let mut expected_collision_map = vec![vec![None; 3]; 3];
        expected_collision_map[2][1] = Some((pusher, SokobanBlock::Dynamic));

        assert_eq!(
            push_collision_map_entry(collision_map, IVec2::new(1, 1), super::Direction::Up),
            (expected_collision_map, Some(vec![pusher]))
        );
    }

    #[test]
    fn push_dynamic_into_static() {
        let pusher = Entity::from_raw(0);
        let wall = Entity::from_raw(1);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[2][1] = Some((pusher, SokobanBlock::Dynamic));
        collision_map[1][1] = Some((wall, SokobanBlock::Static));

        assert_eq!(
            push_collision_map_entry(
                collision_map.clone(),
                IVec2::new(1, 2),
                super::Direction::Down
            ),
            (collision_map, None)
        );
    }

    #[test]
    fn push_dynamic_into_boundary() {
        let pusher = Entity::from_raw(0);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[0][0] = Some((pusher, SokobanBlock::Dynamic));

        assert_eq!(
            push_collision_map_entry(
                collision_map.clone(),
                IVec2::new(0, 0),
                super::Direction::Left
            ),
            (collision_map, None)
        );
    }

    #[test]
    fn push_dynamic_into_dynamic_into_empty() {
        let pusher = Entity::from_raw(0);
        let pushed = Entity::from_raw(1);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[1][0] = Some((pusher, SokobanBlock::Dynamic));
        collision_map[1][1] = Some((pushed, SokobanBlock::Dynamic));

        let mut expected_collision_map = vec![vec![None; 3]; 3];
        expected_collision_map[1][1] = Some((pusher, SokobanBlock::Dynamic));
        expected_collision_map[1][2] = Some((pushed, SokobanBlock::Dynamic));

        assert_eq!(
            push_collision_map_entry(collision_map, IVec2::new(0, 1), super::Direction::Right),
            (expected_collision_map, Some(vec![pushed, pusher]))
        );
    }

    #[test]
    fn push_dynamic_into_dynamic_into_static() {
        let pusher = Entity::from_raw(0);
        let pushed = Entity::from_raw(1);
        let wall = Entity::from_raw(2);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[2][0] = Some((pusher, SokobanBlock::Dynamic));
        collision_map[2][1] = Some((pushed, SokobanBlock::Dynamic));
        collision_map[2][2] = Some((wall, SokobanBlock::Static));

        assert_eq!(
            push_collision_map_entry(
                collision_map.clone(),
                IVec2::new(0, 2),
                super::Direction::Right
            ),
            (collision_map, None)
        );
    }

    #[test]
    fn push_dynamic_into_dynamic_into_boundary() {
        let pusher = Entity::from_raw(0);
        let pushed = Entity::from_raw(1);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[1][1] = Some((pusher, SokobanBlock::Dynamic));
        collision_map[2][1] = Some((pushed, SokobanBlock::Dynamic));

        assert_eq!(
            push_collision_map_entry(
                collision_map.clone(),
                IVec2::new(1, 1),
                super::Direction::Up,
            ),
            (collision_map, None)
        );
    }

    fn app_setup() -> App {
        #[derive(Clone, PartialEq, Eq, Debug, Default, Hash, States)]
        enum State {
            #[default]
            Only,
        }

        let mut app = App::new();

        app.add_state::<State>()
            .add_plugins(SokobanPlugin::new(State::Only, "MyLayerIdentifier"));

        app.world.spawn(LayerMetadata {
            c_wid: 3,
            c_hei: 4,
            grid_size: 32,
            identifier: "MyLayerIdentifier".to_string(),
            ..default()
        });

        app
    }

    #[test]
    fn push_with_sokoban_commands() {
        let mut app = app_setup();

        let block_a = app
            .world
            .spawn((GridCoords::new(1, 1), SokobanBlock::Dynamic))
            .id();
        let block_b = app
            .world
            .spawn((GridCoords::new(1, 2), SokobanBlock::Dynamic))
            .id();
        let block_c = app
            .world
            .spawn((GridCoords::new(2, 2), SokobanBlock::Dynamic))
            .id();

        let mut system_state: SystemState<SokobanCommands> = SystemState::new(&mut app.world);
        let mut sokoban_commands: SokobanCommands = system_state.get_mut(&mut app.world);

        sokoban_commands.move_block(block_a, super::Direction::Up);
        sokoban_commands.move_block(block_c, super::Direction::Left);

        system_state.apply(&mut app.world);

        app.update();

        assert_eq!(
            *app.world.entity(block_a).get::<GridCoords>().unwrap(),
            GridCoords::new(0, 2)
        );
        assert_eq!(
            *app.world.entity(block_b).get::<GridCoords>().unwrap(),
            GridCoords::new(1, 3)
        );
        assert_eq!(
            *app.world.entity(block_c).get::<GridCoords>().unwrap(),
            GridCoords::new(1, 2)
        );
    }

    #[test]
    fn push_tracker_sends_events() {
        let mut app = app_setup();

        let block_a = app
            .world
            .spawn((GridCoords::new(1, 1), SokobanBlock::Dynamic, PushTracker))
            .id();
        let block_b = app
            .world
            .spawn((GridCoords::new(1, 2), SokobanBlock::Dynamic))
            .id();
        let block_c = app
            .world
            .spawn((GridCoords::new(2, 2), SokobanBlock::Dynamic))
            .id();

        let mut system_state: SystemState<SokobanCommands> = SystemState::new(&mut app.world);
        let mut sokoban_commands: SokobanCommands = system_state.get_mut(&mut app.world);

        sokoban_commands.move_block(block_a, super::Direction::Up);
        sokoban_commands.move_block(block_c, super::Direction::Left);

        system_state.apply(&mut app.world);

        app.update();

        let events = app.world.resource::<Events<PushEvent>>();
        let mut reader = events.get_reader();

        assert_eq!(events.len(), 1);
        assert_eq!(
            *reader.read(events).next().unwrap(),
            PushEvent {
                pusher: block_a,
                direction: super::Direction::Up,
                pushed: vec![block_b],
            }
        );
    }
}
