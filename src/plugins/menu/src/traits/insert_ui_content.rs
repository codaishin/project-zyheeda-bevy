use bevy::hierarchy::ChildBuilder;
use common::traits::{handles_localization::LocalizeToken, thread_safe::ThreadSafe};

pub trait InsertUiContent {
	fn insert_ui_content<TLocalization>(
		&self,
		localization: &mut TLocalization,
		parent: &mut ChildBuilder,
	) where
		TLocalization: LocalizeToken + ThreadSafe;
}
