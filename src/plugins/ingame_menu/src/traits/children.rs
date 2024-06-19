use bevy::hierarchy::ChildBuilder;

pub trait Children {
	fn children(&mut self, parent: &mut ChildBuilder);
}
