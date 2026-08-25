use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::prelude::*;

pub trait InsertUiContent {
	fn insert_ui_content<TLocalization>(
		&self,
		localization: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: Localize;
}
