use crate::{
    animation::SpriteSheetAnimation,
    gameplay::{components::MoveTable, Direction},
    gameplay::{xy_translation, *},
    history::{History, HistoryCommands},
    resources::{RewindSettings, RewindTimer},
    sokoban::RigidBody,
    *,
};
use bevy::{prelude::*, utils::Duration};
use bevy_easings::*;

pub struct WilloPlugin;

impl Plugin for WilloPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FromComponentAnimator::<WilloAnimationState>::new())
            .add_event::<WilloMovementEvent>()
            .add_system(
                willo_input
                    .run_in_state(GameState::Gameplay)
                    .label(SystemLabels::Input)
                    .before(history::FlushHistoryCommands),
            )
            .add_system(
                move_willo_by_table
                    .run_in_state(GameState::Gameplay)
                    .after(SystemLabels::MoveTableUpdate)
                    .after(history::FlushHistoryCommands),
            )
            // Systems with potential easing end/beginning collisions cannot be in CoreStage::Update
            // see https://github.com/vleue/bevy_easings/issues/23
            .add_system_to_stage(
                CoreStage::PostUpdate,
                reset_willo_easing
                    .run_not_in_state(GameState::AssetLoading)
                    .before("ease_movement"),
            )
            .add_system(play_death_animations.run_not_in_state(GameState::AssetLoading))
            .add_system(history_sugar.run_not_in_state(GameState::AssetLoading))
            .register_ldtk_entity::<WilloBundle>("Willo");
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct WilloMovementEvent {
    pub direction: Direction,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum WilloState {
    Waiting,
    Dead,
    RankMove(KeyCode),
    FileMove(KeyCode),
}

impl Default for WilloState {
    fn default() -> WilloState {
        WilloState::Waiting
    }
}

const MOVEMENT_SECONDS: f32 = 0.14;

#[derive(Clone, Debug, Component)]
struct MovementTimer(Timer);

impl Default for MovementTimer {
    fn default() -> MovementTimer {
        MovementTimer(Timer::from_seconds(MOVEMENT_SECONDS, false))
    }
}

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

        let frame_timer = Timer::new(Duration::from_millis(150), true);

        let repeat = matches!(state, Idle(Down));

        SpriteSheetAnimation {
            indices,
            frame_timer,
            repeat,
        }
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
                let xy = xy_translation(grid_coords.into());
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
    mut death_event_reader: EventReader<DeathEvent>,
) {
    for DeathEvent { willo_entity } in death_event_reader.iter() {
        if let Ok(mut animation_state) = willo_query.get_mut(*willo_entity) {
            *animation_state = WilloAnimationState::Dying;
        }
    }
}

fn move_willo_by_table(
    table_query: Query<&MoveTable>,
    mut willo_query: Query<(&mut MovementTimer, &mut WilloState)>,
    mut movement_writer: EventWriter<WilloMovementEvent>,
    time: Res<Time>,
) {
    for table in table_query.iter() {
        if let Ok((mut timer, mut willo)) = willo_query.get_single_mut() {
            timer.0.tick(time.delta());

            if timer.0.finished() {
                match *willo {
                    WilloState::RankMove(key) => {
                        for (i, rank) in table.table.iter().enumerate() {
                            if rank.contains(&Some(key)) {
                                movement_writer.send(WilloMovementEvent {
                                    direction: DIRECTION_ORDER[i],
                                });
                            }
                        }
                        *willo = WilloState::FileMove(key);
                        timer.0.reset();
                    }
                    WilloState::FileMove(key) => {
                        for rank in table.table.iter() {
                            for (i, cell) in rank.iter().enumerate() {
                                if *cell == Some(key) {
                                    movement_writer.send(WilloMovementEvent {
                                        direction: DIRECTION_ORDER[i],
                                    });
                                }
                            }
                        }
                        *willo = WilloState::Waiting;
                        timer.0.reset();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn willo_input(
    mut willo_query: Query<&mut WilloState>,
    input: Res<Input<KeyCode>>,
    mut history_commands: EventWriter<HistoryCommands>,
    mut rewind_settings: ResMut<RewindSettings>,
    time: Res<Time>,
) {
    for mut willo in willo_query.iter_mut() {
        if *willo == WilloState::Waiting {
            if input.just_pressed(KeyCode::W) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(KeyCode::W)
            } else if input.just_pressed(KeyCode::A) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(KeyCode::A)
            } else if input.just_pressed(KeyCode::S) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(KeyCode::S)
            } else if input.just_pressed(KeyCode::D) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(KeyCode::D)
            }
        }

        if *willo == WilloState::Waiting || *willo == WilloState::Dead {
            if input.just_pressed(KeyCode::Z) {
                history_commands.send(HistoryCommands::Rewind);
                *willo = WilloState::Waiting;
                rewind_settings.hold_timer =
                    Some(RewindTimer::new(rewind_settings.hold_range_millis.end));
            } else if input.pressed(KeyCode::Z) {
                let range = rewind_settings.hold_range_millis.clone();
                let acceleration = rewind_settings.hold_acceleration;

                if let Some(RewindTimer { velocity, timer }) = &mut rewind_settings.hold_timer {
                    *velocity = (*velocity - (acceleration * time.delta_seconds()))
                        .clamp(range.start as f32, range.end as f32);

                    timer.tick(time.delta());

                    if timer.just_finished() {
                        history_commands.send(HistoryCommands::Rewind);
                        *willo = WilloState::Waiting;

                        timer.set_duration(Duration::from_millis(*velocity as u64));
                    }
                }
            } else if input.just_pressed(KeyCode::R) {
                history_commands.send(HistoryCommands::Reset);
                *willo = WilloState::Waiting;
            }
        }
    }
}
