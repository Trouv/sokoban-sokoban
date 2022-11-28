// these two lints are triggered by normal system code a lot
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod animation;
mod camera;
mod event_scheduler;
mod from_component;
mod graveyard;
mod history;
mod level_select;
mod level_transition;
mod nine_slice;
mod previous_component;
mod ui;
mod wind;

use animation::SpriteSheetAnimationPlugin;
use bevy::prelude::*;

use bevy_asset_loader::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;

pub const UNIT_LENGTH: i32 = 32;

#[cfg(feature = "inspector")]
use bevy_inspector_egui::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    AssetLoading,
    LevelTransition,
    Gameplay,
    LevelSelect,
}

fn main() {
    let level_selection = if std::env::args().count() > 1 {
        let level_arg = std::env::args().last().unwrap();

        match level_arg.parse::<usize>() {
            Ok(num) => LevelSelection::Index(num - 1),
            _ => LevelSelection::Identifier(level_arg),
        }
    } else {
        LevelSelection::Index(0)
    };

    let mut app = App::new();

    app.insert_resource(Msaa { samples: 1 })
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    watch_for_changes: cfg!(feature = "hot"),
                    ..default()
                }),
        )
        .add_plugin(EasingsPlugin)
        .add_plugin(LdtkPlugin)
        .insert_resource(LdtkSettings {
            set_clear_color: SetClearColor::FromEditorBackground,
            ..default()
        })
        .add_loopless_state(GameState::AssetLoading)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::LevelTransition)
                .with_collection::<AssetHolder>(),
        )
        .add_plugin(SpriteSheetAnimationPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(level_select::LevelSelectPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(graveyard::control_display::ControlDisplayPlugin)
        .add_plugin(graveyard::willo::WilloPlugin)
        .add_plugin(graveyard::sokoban::SokobanPlugin)
        .add_plugin(graveyard::movement_table::MovementTablePlugin)
        .add_plugin(graveyard::gravestone::GravestonePlugin)
        .add_plugin(graveyard::goal::GoalPlugin)
        .add_plugin(graveyard::exorcism::ExorcismPlugin)
        .add_plugin(graveyard::wind::WindPlugin)
        .add_plugin(level_transition::LevelTransitionPlugin)
        .insert_resource(level_selection.clone())
        .insert_resource(level_transition::TransitionTo(level_selection));

    #[cfg(feature = "inspector")]
    {
        app.add_plugin(WorldInspectorPlugin::new());
    }

    app.run()
}

#[derive(Debug, Default, AssetCollection, Resource)]
pub struct AssetHolder {
    #[asset(path = "levels/willos-graveyard.ldtk")]
    pub ldtk: Handle<LdtkAsset>,
    #[asset(path = "fonts/WayfarersToyBoxRegular-gxxER.ttf")]
    pub font: Handle<Font>,
    #[asset(path = "textures/button-underline.png")]
    pub button_underline: Handle<Image>,
    #[asset(path = "textures/button-radial.png")]
    pub button_radial: Handle<Image>,
    #[asset(path = "sfx/victory.wav")]
    pub victory_sound: Handle<AudioSource>,
    #[asset(path = "sfx/push.wav")]
    pub push_sound: Handle<AudioSource>,
    #[asset(path = "sfx/undo.wav")]
    pub undo_sound: Handle<AudioSource>,
    #[asset(path = "textures/tarot.png")]
    pub tarot_sheet: Handle<Image>,
}
