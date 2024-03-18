use crate::{
	components::{Light, Wall},
	traits::Definition,
	types::ForChildren,
};

impl Definition<Light<Wall>> for Light<Wall> {
	fn target_names() -> Vec<String> {
		vec![
			"LightNZ".to_owned(),
			"LightNX".to_owned(),
			"LightPZ".to_owned(),
			"LightPX".to_owned(),
		]
	}

	fn bundle() -> (Light<Wall>, ForChildren) {
		(Light::<Wall>::default(), ForChildren::from(true))
	}
}
