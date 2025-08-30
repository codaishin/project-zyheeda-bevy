use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::traits::{handles_localization::Localize, thread_safe::ThreadSafe};

pub trait InsertUiContent {
	fn insert_ui_content<TLocalization>(
		&self,
		localization: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: Localize + ThreadSafe;
}
