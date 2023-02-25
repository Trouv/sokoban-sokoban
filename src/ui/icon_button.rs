use bevy::{prelude::*, ui::FocusPolicy};
use bevy_asset_loader::prelude::AssetCollection;

use crate::{
    ui::text_button::ButtonRadial,
    ui_atlas_image::{AtlasImageBundle, UiAtlasImage},
};

pub struct IconButtonPlugin;

impl Plugin for IconButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_icon_button_elements);
    }
}

#[derive(Default, Debug, Component)]
pub struct IconButton {
    pub icon: UiAtlasImage,
}

#[derive(Default, Debug, Bundle)]
pub struct IconButtonBundle {
    icon_button: IconButton,
    button_bundle: ButtonBundle,
    previous_interaction: PreviousComponent<Interaction>,
}

impl IconButtonBundle {
    fn new(icon: UiAtlasImage, diameter: Val) -> IconButtonBundle {
        IconButtonBundle {
            icon_button: IconButton { icon },
            button_bundle: ButtonBundle {
                style: Style {
                    size: Size {
                        width: diameter,
                        height: diameter,
                    },
                    ..default()
                },
                //interaction: Interaction::None,
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            previous_interaction: PreviousComponent::<Interaction>::default(),
        }
    }
}

#[derive(Default, Debug, AssetCollection, Resource)]
pub struct IconButtonAssets {
    #[asset(path = "textures/icon-button-outline.png")]
    outline: Handle<Image>,
    #[asset(path = "textures/icon-button-radial.png")]
    radial: Handle<Image>,
}

fn spawn_icon_button_elements(
    mut commands: Commands,
    icon_buttons: Query<(Entity, &IconButton), Changed<IconButton>>,
    assets: Res<IconButtonAssets>,
) {
    for (entity, icon_button) in &icon_buttons {
        commands.entity(entity).despawn_descendants();

        commands.entity(entity).add_children(|parent| {
            // Radial
            parent.spawn(ButtonRadial).insert(ImageBundle {
                image: UiImage(assets.radial.clone()),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect::all(Val::Percent(12.5)),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                ..default()
            });

            // Outline
            parent.spawn(ImageBundle {
                image: UiImage(assets.outline.clone()),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect::all(Val::Percent(0.)),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                ..default()
            });

            // Icon
            parent.spawn(AtlasImageBundle {
                atlas_image: icon_button.icon.clone(),
                image_bundle: ImageBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect::all(Val::Percent(0.)),
                        ..default()
                    },
                    focus_policy: FocusPolicy::Pass,
                    ..default()
                },
            });
        });
    }
}
