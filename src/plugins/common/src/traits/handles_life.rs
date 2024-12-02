use bevy::prelude::Component;

pub trait HandlesLife {
	type TLife: Component + ChangeLife;
}

pub trait ChangeLife {
	fn change_by(&mut self, value: f32);
}
