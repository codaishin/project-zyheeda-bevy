use super::accessors::get::GetterRef;
use crate::{attributes::health::Health, traits::handles_saving::SavableComponent};
use bevy::{ecs::component::Mutable, prelude::Component};

pub trait HandlesLife {
	type TLife: Component<Mutability = Mutable>
		+ ChangeLife
		+ GetterRef<Health>
		+ From<Health>
		+ SavableComponent;
}

pub trait ChangeLife {
	fn change_by(&mut self, value: f32);
}
