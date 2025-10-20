use crate::components::player::Player;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::{
	handles_input::GetAllInputStates,
	handles_physics::{Raycast, SolidObjects},
};

impl Player {
	pub(crate) fn update_skill_actions<TInput, TRaycast>()
	where
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputStates>,
		for<'w, 's> TRaycast: SystemParam<Item<'w, 's>: Raycast<SolidObjects>>,
	{
	}
}

#[cfg(test)]
mod tests {
	use super::*;
}
