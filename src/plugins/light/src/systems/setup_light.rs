use bevy::{pbr::light_consts::lux::HALLWAY, prelude::*};

pub(crate) fn setup_light(default_color: Srgba) -> impl Fn(Commands, ResMut<AmbientLight>) {
	move |mut commands, mut ambient_light| {
		*ambient_light = AmbientLight::NONE;
		let light = DirectionalLight {
			shadows_enabled: false,
			illuminance: HALLWAY,
			color: Color::from(default_color),
			..default()
		};

		for x in [-1., 1.] {
			for z in [-1., 1.] {
				commands.spawn((
					Transform::default().looking_to(Vec3::new(x, -1., z), Vec3::Y),
					light.clone(),
				));
			}
		}
	}
}
