//! Plugin, components and events providing functionality for Willo, the player character.
use crate::{
    animation::{FromComponentAnimator, SpriteSheetAnimation},
    graveyard::{
        exorcism::ExorcismEvent,
        gravestone::GraveId,
        movement_table::Direction,
        sokoban::{RigidBody, SokobanLabels},
    },
    history::{History, HistoryCommands, HistoryPlugin},
    AssetHolder, GameState, UNIT_LENGTH,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};
use iyes_loopless::prelude::*;
use std::time::Duration;

/// Labels used by Willo systems.
#[derive(SystemLabel)]
pub enum WilloLabels {
    Input,
}

/// Plugin providing functionality for Willo, the player character.
pub struct WilloPlugin;

impl Plugin for WilloPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FromComponentAnimator::<WilloAnimationState>::new())
            .add_plugin(HistoryPlugin::<GridCoords, _>::run_in_state(
                GameState::Graveyard,
            ))
            .add_event::<WilloMovementEvent>()
            // Systems with potential easing end/beginning collisions cannot be in CoreStage::Update
            // see https://github.com/vleue/bevy_easings/issues/23
            .add_system_to_stage(
                CoreStage::PostUpdate,
                reset_willo_easing
                    .run_not_in_state(GameState::AssetLoading)
                    .before(SokobanLabels::EaseMovement),
            )
            .add_system(play_death_animations.run_not_in_state(GameState::AssetLoading))
            .add_system(history_sugar.run_not_in_state(GameState::AssetLoading))
            .register_ldtk_entity::<WilloBundle>("Willo");
    }
}

/// Event that fires whenever Willo moves.
///
/// Only fires once per direction - so it fires twice during most moves.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct WilloMovementEvent {
    pub direction: Direction,
}

/// Component that marks Willo and keeps track of their state.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum WilloState {
    Waiting,
    Dead,
    RankMove(GraveId),
    FileMove(GraveId),
}

impl Default for WilloState {
    fn default() -> WilloState {
        WilloState::Waiting
    }
}

/// Component enumerating the possible states of Willo's animation.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Component)]
pub enum WilloAnimationState {
    Idle(Direction),
    Push(Direction),
    Dying,
    None,
}

impl Default for WilloAnimationState {
    fn default() -> Self {
        WilloAnimationState::Idle(Direction::Down)
    }
}

impl Iterator for WilloAnimationState {
    type Item = Self;
    fn next(&mut self) -> Option<Self::Item> {
        Some(match self {
            WilloAnimationState::Dying | WilloAnimationState::None => WilloAnimationState::None,
            WilloAnimationState::Push(d) => WilloAnimationState::Idle(*d),
            _ => WilloAnimationState::Idle(Direction::Down),
        })
    }
}

impl From<WilloAnimationState> for SpriteSheetAnimation {
    fn from(state: WilloAnimationState) -> SpriteSheetAnimation {
        use Direction::*;
        use WilloAnimationState::*;

        let indices = match state {
            Push(Up) => 1..2,
            Push(Down) => 11..12,
            Push(Left) => 21..22,
            Push(Right) => 31..32,
            Idle(Up) => 40..47,
            Idle(Down) => 50..57,
            Idle(Left) => 60..67,
            Idle(Right) => 70..77,
            Dying => 80..105,
            None => 3..4,
        };

        let frame_timer = Timer::new(Duration::from_millis(150), TimerMode::Repeating);

        let repeat = matches!(state, Idle(Down));

        SpriteSheetAnimation {
            indices,
            frame_timer,
            repeat,
        }
    }
}

const MOVEMENT_SECONDS: f32 = 0.14;

/// Component that provides the timer used to space out the movements Willo performs.
#[derive(Clone, Debug, Component)]
pub struct MovementTimer(pub Timer);

impl Default for MovementTimer {
    fn default() -> MovementTimer {
        MovementTimer(Timer::from_seconds(MOVEMENT_SECONDS, TimerMode::Once))
    }
}

#[derive(Clone, Bundle, LdtkEntity)]
struct WilloBundle {
    #[grid_coords]
    grid_coords: GridCoords,
    history: History<GridCoords>,
    #[from_entity_instance]
    rigid_body: RigidBody,
    willo_state: WilloState,
    movement_timer: MovementTimer,
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
    willo_animation_state: WilloAnimationState,
}

fn reset_willo_easing(
    mut commands: Commands,
    willo_query: Query<
        (Entity, &GridCoords, &Transform, &WilloAnimationState),
        Changed<WilloAnimationState>,
    >,
) {
    if let Ok((entity, &grid_coords, transform, animation_state)) = willo_query.get_single() {
        match animation_state {
            WilloAnimationState::Push(_) => (),
            _ => {
                let xy = grid_coords_to_translation(grid_coords, IVec2::splat(UNIT_LENGTH));
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
}

fn history_sugar(
    mut history_commands: EventReader<HistoryCommands>,
    mut willo_query: Query<&mut WilloAnimationState>,
    audio: Res<Audio>,
    sfx: Res<AssetHolder>,
) {
    for command in history_commands.iter() {
        match command {
            HistoryCommands::Rewind | HistoryCommands::Reset => {
                *willo_query.single_mut() = WilloAnimationState::Idle(Direction::Down);
                audio.play(sfx.undo_sound.clone_weak());
            }
            _ => (),
        }
    }
}

fn play_death_animations(
    mut willo_query: Query<&mut WilloAnimationState>,
    mut death_event_reader: EventReader<ExorcismEvent>,
) {
    for ExorcismEvent { willo_entity } in death_event_reader.iter() {
        if let Ok(mut animation_state) = willo_query.get_mut(*willo_entity) {
            *animation_state = WilloAnimationState::Dying;
        }
    }
}
