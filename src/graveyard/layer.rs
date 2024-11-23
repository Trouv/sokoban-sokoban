use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0:?} layer entity does not exist.")]
pub struct LayerEntityDoesNotExist(GraveyardLayer);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GraveyardLayer {
    Background,
    BorderBackground,
    BackgroundEntities,
    IntGrid,
    Entities,
    Foreground,
}

impl GraveyardLayer {
    const BACKGROUND_IDENTIFIER: &'static str = "Background";
    const BORDER_BACKGROUND_IDENTIFIER: &'static str = "BorderBG";
    const BACKGROUND_ENTITIES_IDENTIFIER: &'static str = "Background_Entities";
    const INT_GRID_IDENTIFIER: &'static str = "IntGrid";
    const ENTITIES_IDENTIFIER: &'static str = "Entities";
    const FOREGROUND_IDENTIFIER: &'static str = "ForeGround";

    fn as_identifier(&self) -> &'static str {
        match self {
            GraveyardLayer::Background => Self::BACKGROUND_IDENTIFIER,
            GraveyardLayer::BorderBackground => Self::BORDER_BACKGROUND_IDENTIFIER,
            GraveyardLayer::BackgroundEntities => Self::BACKGROUND_ENTITIES_IDENTIFIER,
            GraveyardLayer::IntGrid => Self::INT_GRID_IDENTIFIER,
            GraveyardLayer::Entities => Self::ENTITIES_IDENTIFIER,
            GraveyardLayer::Foreground => Self::FOREGROUND_IDENTIFIER,
        }
    }

    pub fn spawn_bundles_on<I>(
        self,
    ) -> impl Fn(In<I>, Commands, Query<(Entity, &LayerMetadata)>) -> Result<(), LayerEntityDoesNotExist>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: Bundle,
    {
        move |In(bundles_iter), mut commands, layer_query| {
            let (layer_entity, _) = layer_query
                .iter()
                .find(|(_, metadata)| metadata.identifier == self.as_identifier())
                .ok_or(LayerEntityDoesNotExist(self))?;

            bundles_iter.into_iter().for_each(|bundle| {
                commands.spawn(bundle).set_parent(layer_entity);
            });

            Ok(())
        }
    }
}
