use super::accessors::get::GetterRef;
use crate::attributes::health::Health;
use bevy::{ecs::component::Mutable, prelude::Component};

pub trait HandlesLife {
	type TLife: Component<Mutability = Mutable> + ChangeLife + GetterRef<Health> + From<Health>;
}

pub trait ChangeLife {
	fn change_by(&mut self, value: f32);
}
