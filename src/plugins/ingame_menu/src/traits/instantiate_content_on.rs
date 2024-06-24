use bevy::hierarchy::ChildBuilder;

pub trait InstantiateContentOn {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder);
}
