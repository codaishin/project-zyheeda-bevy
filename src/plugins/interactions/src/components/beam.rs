use super::{RayCasterArgs, RayFilter};
use crate::events::{InteractionEvent, Ray};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	components::GroundOffset,
	tools::Units,
	traits::{
		cast_ray::TimeOfImpact,
		handles_interactions::BeamParameters,
		handles_lifetime::HandlesLifetime,
		try_despawn::TryDespawn,
		try_insert_on::TryInsertOn,
	},
};
use std::time::Duration;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility)]
pub(crate) struct Beam {
	source: Vec3,
	target: Vec3,
}

impl Beam {
	pub(crate) fn execute<TLifetimes>(
		mut commands: Commands,
		mut ray_cast_events: EventReader<InteractionEvent<Ray>>,
		beams: Query<(Entity, &BeamCommand, Option<&Beam>)>,
		transforms: Query<(&GlobalTransform, Option<&GroundOffset>)>,
	) where
		TLifetimes: HandlesLifetime,
	{
		for (entity, cmd, ..) in &beams {
			match commands.get_entity(cmd.source) {
				Ok(_) => defer_beam_ray_cast(&mut commands, &transforms, entity, cmd),
				Err(_) => despawn_beam(&mut commands, entity),
			}
		}

		for InteractionEvent(source, ray) in ray_cast_events.read() {
			match beams.get(*source) {
				Ok((entity, cmd, None)) => {
					spawn_beam::<TLifetimes>(&mut commands, entity, ray, cmd)
				}
				Ok((entity, .., Some(_))) => update_beam_transform(&mut commands, entity, ray),
				Err(_) => {}
			}
		}
	}
}

#[derive(Component, Debug, PartialEq)]
pub struct BeamCommand {
	source: Entity,
	target: Entity,
	params: Parameters,
}

impl<T> From<&T> for BeamCommand
where
	T: BeamParameters,
{
	fn from(value: &T) -> Self {
		BeamCommand {
			source: value.source(),
			target: value.target(),
			params: Parameters {
				range: value.range(),
				lifetime: value.lifetime(),
			},
		}
	}
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct Parameters {
	range: Units,
	lifetime: Duration,
}

fn defer_beam_ray_cast(
	commands: &mut Commands,
	transforms: &Query<(&GlobalTransform, Option<&GroundOffset>)>,
	entity: Entity,
	cmd: &BeamCommand,
) {
	let Ok((source_transform, source_offset)) = transforms.get(cmd.source) else {
		return;
	};
	let Ok((target_transform, target_offset)) = transforms.get(cmd.target) else {
		return;
	};
	let Some(filter) = get_filter(cmd.source) else {
		return;
	};
	let origin = translation(source_transform, source_offset);
	let target = translation(target_transform, target_offset);
	let Ok(direction) = Dir3::new(target - origin) else {
		return;
	};

	commands.try_insert_on(
		entity,
		RayCasterArgs {
			origin,
			direction,
			solid: true,
			filter,
			max_toi: TimeOfImpact(*cmd.params.range),
		},
	);
}

fn despawn_beam(commands: &mut Commands, entity: Entity) {
	commands.try_despawn(entity)
}

fn get_filter(source: Entity) -> Option<RayFilter> {
	QueryFilter::default()
		.exclude_rigid_body(source)
		.try_into()
		.ok()
}

fn spawn_beam<TLifetimes>(commands: &mut Commands, entity: Entity, ray: &Ray, cmd: &BeamCommand)
where
	TLifetimes: HandlesLifetime,
{
	let (source, target, transform) = unpack_beam_ray(ray);
	commands.try_insert_on(
		entity,
		(
			transform,
			Beam { source, target },
			TLifetimes::lifetime(cmd.params.lifetime),
		),
	);
}

fn update_beam_transform(commands: &mut Commands, entity: Entity, ray: &Ray) {
	let (.., transform) = unpack_beam_ray(ray);
	commands.try_insert_on(entity, transform);
}

fn translation(transform: &GlobalTransform, offset: Option<&GroundOffset>) -> Vec3 {
	transform.translation() + offset.map_or(Vec3::ZERO, |offset| offset.0)
}

type SourceTranslation = Vec3;
type TargetTranslation = Vec3;

fn get_beam_range(ray: &Ray3d, toi: f32) -> (SourceTranslation, TargetTranslation) {
	(ray.origin, ray.origin + *ray.direction * toi)
}

fn unpack_beam_ray(
	Ray(ray, TimeOfImpact(toi)): &Ray,
) -> (SourceTranslation, TargetTranslation, Transform) {
	let (source, target) = get_beam_range(ray, *toi);

	(
		source,
		target,
		Transform::from_translation((source + target) / 2.)
			.looking_at(target, Vec3::Y)
			.with_scale(Vec3::ONE.with_z(*toi)),
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::RayCasterArgs,
		events::{InteractionEvent, Ray},
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{cast_ray::TimeOfImpact, clamp_zero_positive::ClampZeroPositive},
	};
	use std::time::Duration;

	struct _HandlesLifetime;

	impl HandlesLifetime for _HandlesLifetime {
		fn lifetime(duration: Duration) -> impl Bundle {
			_LifeTime(duration)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _LifeTime(Duration);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, Beam::execute::<_HandlesLifetime>);
		app.add_event::<InteractionEvent<Ray>>();

		app
	}

	#[test]
	fn insert_ray_caster() {
		let mut app = setup();
		let source = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 0.))
			.id();
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 4.))
			.id();
		let beam = app
			.world_mut()
			.spawn(BeamCommand {
				source,
				target,
				params: Parameters {
					range: Units::new(100.),
					..default()
				},
			})
			.id();

		app.update();

		assert_eq!(
			Some(&RayCasterArgs {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Z,
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(source)
					.try_into()
					.unwrap(),
			}),
			app.world().entity(beam).get::<RayCasterArgs>()
		);
	}

	#[test]
	fn insert_ray_caster_with_ground_offset_for_target() {
		let mut app = setup();
		let source = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 0.))
			.id();
		let target = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 4.),
				GroundOffset(Vec3::new(0., 1., 0.)),
			))
			.id();
		let beam = app
			.world_mut()
			.spawn(BeamCommand {
				source,
				target,
				params: Parameters {
					range: Units::new(100.),
					..default()
				},
			})
			.id();

		app.update();

		assert_eq!(
			Some(&RayCasterArgs {
				origin: Vec3::new(1., 0., 0.),
				direction: Vec3::new(0., 1., 4.).normalize().try_into().unwrap(),
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(source)
					.try_into()
					.unwrap(),
			}),
			app.world().entity(beam).get::<RayCasterArgs>()
		);
	}

	#[test]
	fn insert_ray_caster_with_ground_offset_for_source() {
		let mut app = setup();
		let source = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.),
				GroundOffset(Vec3::new(0., 1., 0.)),
			))
			.id();
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 4.))
			.id();
		let beam = app
			.world_mut()
			.spawn(BeamCommand {
				source,
				target,
				params: Parameters {
					range: Units::new(100.),
					..default()
				},
			})
			.id();

		app.update();

		assert_eq!(
			Some(&RayCasterArgs {
				origin: Vec3::new(1., 1., 0.),
				direction: Vec3::new(0., -1., 4.).normalize().try_into().unwrap(),
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(source)
					.try_into()
					.unwrap(),
			}),
			app.world().entity(beam).get::<RayCasterArgs>()
		);
	}

	#[test]
	fn spawn_beam_from_interaction() {
		let mut app = setup();
		let source = app.world_mut().spawn(GlobalTransform::default()).id();
		let beam = app
			.world_mut()
			.spawn(BeamCommand {
				source,
				target: Entity::from_raw(default()),
				params: Parameters {
					lifetime: Duration::from_millis(100),
					..default()
				},
			})
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::Z,
				direction: Dir3::Y,
			},
			TimeOfImpact(10.),
		));

		app.update();

		assert_eq!(
			(
				Some(&Beam {
					source: Vec3::Z,
					target: Vec3::new(0., 10., 1.),
				}),
				Some(&_LifeTime(Duration::from_millis(100))),
			),
			(
				app.world().entity(beam).get::<Beam>(),
				app.world().entity(beam).get::<_LifeTime>()
			)
		);
	}

	#[test]
	fn set_spatial_components() {
		let mut app = setup();
		let source = app.world_mut().spawn(GlobalTransform::default()).id();
		let beam = app
			.world_mut()
			.spawn(BeamCommand {
				source,
				target: Entity::from_raw(default()),
				params: Parameters::default(),
			})
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(0., 1., 0.),
				direction: Dir3::X,
			},
			TimeOfImpact(10.),
		));

		app.update();

		assert_eq!(
			(
				Some(
					&Transform::from_xyz(5., 1., 0.)
						.looking_at(Vec3::new(10., 1., 0.), Vec3::Y)
						.with_scale(Vec3 {
							x: 1.,
							y: 1.,
							z: 10.
						})
				),
				Some(&Visibility::default()),
			),
			(
				app.world().entity(beam).get::<Transform>(),
				app.world().entity(beam).get::<Visibility>(),
			)
		);
	}

	#[test]
	fn update_transform_only_when_beam_component_already_present() {
		let mut app = setup();
		let source = app.world_mut().spawn(GlobalTransform::default()).id();
		let beam = app
			.world_mut()
			.spawn((
				BeamCommand {
					source,
					target: Entity::from_raw(default()),
					params: Parameters::default(),
				},
				Beam {
					source: Vec3::default(),
					target: Vec3::default(),
				},
			))
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(0., 1., 0.),
				direction: Dir3::X,
			},
			TimeOfImpact(10.),
		));

		app.update();

		assert_eq!(
			(
				Some(
					&Transform::from_xyz(5., 1., 0.)
						.looking_at(Vec3::new(10., 1., 0.), Vec3::Y)
						.with_scale(Vec3 {
							x: 1.,
							y: 1.,
							z: 10.
						})
				),
				Some(&Beam {
					source: Vec3::default(),
					target: Vec3::default(),
				})
			),
			(
				app.world().entity(beam).get::<Transform>(),
				app.world().entity(beam).get::<Beam>(),
			),
		)
	}

	#[test]
	fn remove_beam_when_source_not_removed() {
		let mut app = setup();
		let source = app.world_mut().spawn(GlobalTransform::default()).id();
		let beam = app
			.world_mut()
			.spawn(BeamCommand {
				source,
				target: Entity::from_raw(default()),
				params: Parameters::default(),
			})
			.id();
		let child = app.world_mut().spawn(ChildOf(beam)).id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(0., 1., 0.),
				direction: Dir3::X,
			},
			TimeOfImpact(10.),
		));

		app.update();
		app.world_mut().entity_mut(source).despawn();
		app.update();

		let beam = app.world().get_entity(beam);
		let child = app.world().get_entity(child);

		assert_eq!((false, false), (beam.is_ok(), child.is_ok()));
	}
}
