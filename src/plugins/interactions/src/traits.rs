use bevy::prelude::{Bundle, Component};

pub(crate) mod damage_health;
pub(crate) mod rapier_context;

pub trait ActOn<TTarget> {
	fn act_on(&mut self, target: &mut TTarget);
}

pub trait ConcatBlockers {
	fn and<TBlocker: Component>(self) -> impl ConcatBlockers + Bundle;
}
