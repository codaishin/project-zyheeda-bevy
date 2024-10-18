use super::SubModelNames;
use common::components::Player;

impl SubModelNames for Player {
	fn sub_model_names() -> Vec<&'static str> {
		vec![
			"ArmTopLeftModel",
			"ArmTopRightModel",
			"ArmBottomLeftModel",
			"ArmBottomRightModel",
		]
	}
}
