use crate::components::global_light::GlobalLight;
use bevy::prelude::*;

impl GlobalLight {
	pub(crate) fn spawn(default_color: Srgba) -> impl Fn(Commands, ResMut<AmbientLight>) {
		move |mut commands, mut ambient_light| {
			*ambient_light = AmbientLight::NONE;

			for x in [-1., 1.] {
				for z in [-1., 1.] {
					commands.spawn((
						Transform::default().looking_to(Vec3::new(x, -1., z), Vec3::Y),
						Self(default_color),
					));
				}
			}
		}
	}
}
