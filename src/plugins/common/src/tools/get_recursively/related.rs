use bevy::prelude::Entity;

pub(crate) struct Child(pub(crate) Entity);

impl Child {
	pub(crate) fn new(entity: Entity) -> Self {
		Child(entity)
	}
}

impl From<Child> for Entity {
	fn from(value: Child) -> Self {
		value.0
	}
}
