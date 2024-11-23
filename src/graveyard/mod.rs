//! Plugin providing logic for all graveyard entities and the entire graveyard state.
//!
//! So, the logic for core gameplay lives here.

pub mod arrow_block;
pub mod control_display;
pub mod exorcism;
pub mod goal;
pub mod gravestone;
pub mod gravestone_movement_queries;
pub mod layer;
pub mod out_of_bounds;
pub mod volatile;
pub mod wall;
pub mod willo;
pub mod wind;

use crate::{
    history::{FlushHistoryCommands, HistoryCommands},
    sokoban,
    ui::{action::UiActionPlugin, button_prompt::ButtonPromptPlugin},
    GameState,
};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, ops::Range, time::Duration};

/// Plugin providing logic for all graveyard entities and the entire graveyard state.
///
/// So, the logic for core gameplay lives here.
pub struct GraveyardPlugin;

impl Plugin for GraveyardPlugin {
    fn build(&self, app: &mut App) {
        let asset_folder = app.get_added_plugins::<AssetPlugin>()[0].file_path.clone();

        app.init_resource::<RewindSettings>()
            .add_plugins((
                InputManagerPlugin::<GraveyardAction>::default(),
                UiActionPlugin::<GraveyardAction>::new(),
                ButtonPromptPlugin::<GraveyardAction>::new(),
            ))
            .init_resource::<ActionState<GraveyardAction>>()
            .insert_resource(
                load_graveyard_control_settings(asset_folder)
                    .expect("unable to load gameplay control settings"),
            )
            .add_plugins((
                control_display::ControlDisplayPlugin,
                willo::WilloPlugin,
                sokoban::SokobanPlugin::<GameState, sokoban::SokobanBlock, sokoban::Direction>::new(
                    GameState::Graveyard,
                    "IntGrid",
                ),
                gravestone::GravestonePlugin,
                volatile::VolatilePlugin,
                wall::WallPlugin,
                goal::GoalPlugin,
                exorcism::ExorcismPlugin,
                wind::WindPlugin,
                arrow_block::ArrowBlockPlugin,
                out_of_bounds::OutOfBoundsPlugin,
            ))
            .add_systems(
                Update,
                graveyard_input
                    .run_if(in_state(GameState::Graveyard))
                    .in_set(willo::WilloSets::Input)
                    .before(FlushHistoryCommands),
            );
    }
}

/// Actions other than grave-actions that can be performed during the gameplay state.
#[derive(Actionlike, Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, Reflect)]
pub enum GraveyardAction {
    /// Undo the last grave-action or restart.
    Undo,
    /// Restart the level to its initial state.
    Restart,
    /// Pause the graveyard state and open up the pause menu.
    Pause,
}

fn load_graveyard_control_settings(
    asset_folder: String,
) -> std::io::Result<InputMap<GraveyardAction>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(serde_json::from_reader(BufReader::new(File::open(
            format!("{asset_folder}/../settings/graveyard_controls.json"),
        )?))?)
    }

    // placed in a `#[cfg]` block rather than `if cfg!` so that changes to the file don't
    // recompile non-wasm builds.
    #[cfg(target_arch = "wasm32")]
    {
        Ok(serde_json::from_str(include_str!(
            "../../settings/graveyard_controls.json"
        ))?)
    }
}

/// Part of the [RewindSettings] resource.
///
/// Provides space between rewinds and tracking rewind velocity for acceleration.
#[derive(Clone, Debug, Default)]
struct RewindTimer {
    velocity: f32,
    timer: Timer,
}

impl RewindTimer {
    fn new(millis: u64) -> RewindTimer {
        RewindTimer {
            velocity: millis as f32,
            timer: Timer::new(Duration::from_millis(millis), TimerMode::Repeating),
        }
    }
}

/// Resource defining the behavior of the rewind feature and storing its state for acceleration.
#[derive(Clone, Debug, Resource)]
struct RewindSettings {
    hold_range_millis: Range<u64>,
    hold_acceleration: f32,
    hold_timer: Option<RewindTimer>,
}

impl Default for RewindSettings {
    fn default() -> Self {
        RewindSettings {
            hold_range_millis: 50..200,
            hold_acceleration: 50.,
            hold_timer: None,
        }
    }
}

fn graveyard_input(
    mut willo_query: Query<&mut willo::WilloState>,
    gameplay_input: Res<ActionState<GraveyardAction>>,
    mut history_commands: EventWriter<HistoryCommands>,
    mut rewind_settings: ResMut<RewindSettings>,
    time: Res<Time>,
) {
    for mut willo in willo_query.iter_mut() {
        if *willo == willo::WilloState::Waiting || *willo == willo::WilloState::Dead {
            if gameplay_input.just_pressed(GraveyardAction::Undo) {
                history_commands.send(HistoryCommands::Rewind);
                *willo = willo::WilloState::Waiting;
                rewind_settings.hold_timer =
                    Some(RewindTimer::new(rewind_settings.hold_range_millis.end));
            } else if gameplay_input.pressed(GraveyardAction::Undo) {
                let range = rewind_settings.hold_range_millis.clone();
                let acceleration = rewind_settings.hold_acceleration;

                if let Some(RewindTimer { velocity, timer }) = &mut rewind_settings.hold_timer {
                    *velocity = (*velocity - (acceleration * time.delta_seconds()))
                        .clamp(range.start as f32, range.end as f32);

                    timer.tick(time.delta());

                    if timer.just_finished() {
                        history_commands.send(HistoryCommands::Rewind);
                        *willo = willo::WilloState::Waiting;

                        timer.set_duration(Duration::from_millis(*velocity as u64));
                    }
                }
            } else if gameplay_input.just_pressed(GraveyardAction::Restart) {
                history_commands.send(HistoryCommands::Reset);
                *willo = willo::WilloState::Waiting;
            }
        }
    }
}
