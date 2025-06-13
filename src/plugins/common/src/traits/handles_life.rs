use super::accessors::get::GetterRef;
use crate::attributes::health::Health;
use bevy::{ecs::component::Mutable, prelude::Component};
use serde::{Serialize, de::DeserializeOwned};

pub trait HandlesLife {
	type TLife: Component<Mutability = Mutable>
		+ ChangeLife
		+ GetterRef<Health>
		+ From<Health>
		+ Clone
		+ Serialize
		+ DeserializeOwned;
}

pub trait ChangeLife {
	fn change_by(&mut self, value: f32);
}
