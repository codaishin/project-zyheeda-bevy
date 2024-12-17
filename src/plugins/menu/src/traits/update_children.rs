use bevy::hierarchy::ChildBuilder;

pub trait UpdateChildren {
	fn update_children(&self, parent: &mut ChildBuilder);
}
