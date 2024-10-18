use crate::traits::entity_names::EntityNames;
use common::components::Player;
use std::marker::PhantomData;

pub(crate) struct SubModels<T>(PhantomData<T>);

impl EntityNames for SubModels<Player> {
	fn entity_names() -> Vec<&'static str> {
		vec![
			"ArmTopLeftModel",
			"ArmTopRightModel",
			"ArmBottomLeftModel",
			"ArmBottomRightModel",
		]
	}
}
