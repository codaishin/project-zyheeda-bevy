use bevy::hierarchy::ChildBuilder;

pub trait Children {
	fn children(&self, parent: &mut ChildBuilder);
}
