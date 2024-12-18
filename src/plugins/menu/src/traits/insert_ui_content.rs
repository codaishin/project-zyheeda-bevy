use bevy::hierarchy::ChildBuilder;

pub trait InsertUiContent {
	fn insert_ui_content(&self, parent: &mut ChildBuilder);
}
