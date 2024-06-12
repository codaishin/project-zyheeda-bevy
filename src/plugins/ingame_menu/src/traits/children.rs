use bevy::hierarchy::ChildBuilder;

pub trait Children {
	fn children(parent: &mut ChildBuilder);
}
