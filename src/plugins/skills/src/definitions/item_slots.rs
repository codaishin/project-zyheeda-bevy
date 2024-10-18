use crate::traits::entity_names::EntityNames;
use common::components::Player;
use std::marker::PhantomData;

pub(crate) struct HandSlots<T>(PhantomData<T>);

impl EntityNames for HandSlots<Player> {
	fn entity_names() -> Vec<&'static str> {
		vec![
			"top_hand_slot.L",
			"top_hand_slot.R",
			"bottom_hand_slot.L",
			"bottom_hand_slot.R",
		]
	}
}

pub(crate) struct ForearmSlots<T>(PhantomData<T>);

impl EntityNames for ForearmSlots<Player> {
	fn entity_names() -> Vec<&'static str> {
		vec![
			"top_forearm.L",
			"top_forearm.R",
			"bottom_forearm.L",
			"bottom_forearm.R",
		]
	}
}
