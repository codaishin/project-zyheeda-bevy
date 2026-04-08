use crate::components::target::Target;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::{Get, GetContextMut},
		handles_animations::{Animations, DirForwardPitch, ForwardPitch, GetForwardPitchMut},
		handles_physics::{MouseHover, MouseHoversOver, Raycast},
		handles_skill_physics::SkillTarget,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::f32::consts::FRAC_PI_2;

impl Target {
	pub(crate) fn update_pitch<TRayCast, TAnimations>(
		mut animations: StaticSystemParam<TAnimations>,
		mut ray_caster: StaticSystemParam<TRayCast>,
		targets: Query<(Entity, &Self, &GlobalTransform), Changed<Self>>,
		transforms: Query<&GlobalTransform>,
		commands: ZyheedaCommands,
	) where
		for<'w, 's> TRayCast: SystemParam<Item<'w, 's>: Raycast<MouseHover>>,
		for<'c> TAnimations:
			SystemParam + GetContextMut<Animations, TContext<'c>: GetForwardPitchMut>,
	{
		for (entity, target, transform) in targets {
			let key = Animations { entity };
			let Some(mut ctx) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};
			let pitch = target.get_pitch(entity, transform, transforms, &commands, &mut ray_caster);

			*ctx.get_forward_pitch_mut() = pitch;
		}
	}

	fn get_pitch(
		&self,
		entity: Entity,
		transform: &GlobalTransform,
		transforms: Query<&GlobalTransform>,
		commands: &ZyheedaCommands,
		ray_cast: &mut impl Raycast<MouseHover>,
	) -> Option<DirForwardPitch> {
		match self.0.as_ref()? {
			SkillTarget::Entity(entity) => {
				let target = commands.get(entity)?;
				let target = transforms.get(target).ok()?.translation();
				get_pitch(transform, target)
			}
			SkillTarget::Cursor => match ray_cast.raycast(MouseHover::excluding([entity]))? {
				MouseHoversOver::Terrain { point } => get_pitch(transform, point),
				MouseHoversOver::Object { entity, .. } => {
					let target = transforms.get(entity).ok()?.translation();
					get_pitch(transform, target)
				}
			},
		}
	}
}

fn get_pitch(transform: &GlobalTransform, to: Vec3) -> Option<DirForwardPitch> {
	let dir = (to - transform.translation()).try_normalize()?;
	let pitch = ForwardPitch::try_from((dir.y.asin() / FRAC_PI_2).abs()).ok()?;

	if dir.y > 0. {
		Some(DirForwardPitch::Up(pitch))
	} else {
		Some(DirForwardPitch::Down(pitch))
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use common::{
		CommonPlugin,
		components::persistent_entity::PersistentEntity,
		traits::{
			handles_animations::{ForwardPitch, GetForwardPitch},
			handles_skill_physics::SkillTarget,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use test_case::test_case;
	use testing::{ApproxEqual, IsChanged, NestedMocks, SingleThreadedApp, assert_eq_approx};

	#[derive(Resource, NestedMocks)]
	struct _RayCast {
		mock: Mock_RayCast,
	}

	impl Default for _RayCast {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_raycast().return_const(None);
			})
		}
	}

	#[automock]
	impl Raycast<MouseHover> for _RayCast {
		fn raycast(&mut self, args: MouseHover) -> Option<MouseHoversOver> {
			self.mock.raycast(args)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Animations {
		forward_pitch: Option<DirForwardPitch>,
	}

	impl ApproxEqual<f32> for _Animations {
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			match (&self.forward_pitch, &other.forward_pitch) {
				(None, None) => true,
				(Some(DirForwardPitch::Up(l)), Some(DirForwardPitch::Up(r))) => {
					l.approx_equal(r, tolerance)
				}
				(Some(DirForwardPitch::Down(l)), Some(DirForwardPitch::Down(r))) => {
					l.approx_equal(r, tolerance)
				}
				_ => false,
			}
		}
	}

	impl GetForwardPitch for _Animations {
		fn get_forward_pitch(&self) -> Option<DirForwardPitch> {
			self.forward_pitch
		}
	}

	impl GetForwardPitchMut for _Animations {
		fn get_forward_pitch_mut(&mut self) -> &mut Option<DirForwardPitch> {
			&mut self.forward_pitch
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin);
		app.init_resource::<_RayCast>();
		app.add_systems(
			Update,
			(
				Target::update_pitch::<ResMut<_RayCast>, Query<&mut _Animations>>,
				IsChanged::<_Animations>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn set_none() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Target(None),
				GlobalTransform::default(),
				_Animations {
					forward_pitch: Some(DirForwardPitch::Down(ForwardPitch::MAX)),
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations {
				forward_pitch: None
			}),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test_case(0., None; "0 degrees")]
	#[test_case(45., ForwardPitch::try_from(0.5).ok().map(DirForwardPitch::Up); "45 degrees up")]
	#[test_case(-45., ForwardPitch::try_from(0.5).ok().map(DirForwardPitch::Down); "45 degrees down")]
	#[test_case(90., DirForwardPitch::Up(ForwardPitch::MAX); "90 degrees up")]
	#[test_case(-90., DirForwardPitch::Down(ForwardPitch::MAX); "90 degrees down")]
	fn set_target_entity_pitch(angle: f32, forward_pitch: impl Into<Option<DirForwardPitch>>) {
		let mut app = setup();
		let translation = Vec3::new(10., 2., 0.);
		let offset = Quat::from_rotation_x(angle.to_radians()).mul_vec3(Vec3::new(0., 0., -30.));
		let target_entity = PersistentEntity::default();
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Entity(target_entity))),
				GlobalTransform::from_translation(translation),
				_Animations {
					forward_pitch: None,
				},
			))
			.id();

		app.world_mut().spawn((
			target_entity,
			GlobalTransform::from_translation(translation + offset),
		));

		app.update();

		assert_eq_approx!(
			Some(&_Animations {
				forward_pitch: forward_pitch.into()
			}),
			app.world().entity(entity).get::<_Animations>(),
			1e-5,
		);
	}

	#[test_case(0., None; "0 degrees")]
	#[test_case(45., ForwardPitch::try_from(0.5).ok().map(DirForwardPitch::Up); "45 degrees up")]
	#[test_case(-45., ForwardPitch::try_from(0.5).ok().map(DirForwardPitch::Down); "45 degrees down")]
	#[test_case(90., DirForwardPitch::Up(ForwardPitch::MAX); "90 degrees up")]
	#[test_case(-90., DirForwardPitch::Down(ForwardPitch::MAX); "90 degrees down")]
	fn set_cursor_terrain_hit_pitch(angle: f32, forward_pitch: impl Into<Option<DirForwardPitch>>) {
		let mut app = setup();
		let translation = Vec3::new(10., 2., 0.);
		let offset = Quat::from_rotation_x(angle.to_radians()).mul_vec3(Vec3::new(0., 0., -30.));
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Cursor)),
				GlobalTransform::from_translation(translation),
				_Animations {
					forward_pitch: None,
				},
			))
			.id();
		app.insert_resource(_RayCast::new().with_mock(|mock| {
			mock.expect_raycast()
				.return_const(Some(MouseHoversOver::Terrain {
					point: translation + offset,
				}));
		}));

		app.update();

		assert_eq_approx!(
			Some(&_Animations {
				forward_pitch: forward_pitch.into()
			}),
			app.world().entity(entity).get::<_Animations>(),
			1e-5,
		);
	}

	#[test_case(0., None; "0 degrees")]
	#[test_case(45., ForwardPitch::try_from(0.5).ok().map(DirForwardPitch::Up); "45 degrees up")]
	#[test_case(-45., ForwardPitch::try_from(0.5).ok().map(DirForwardPitch::Down); "45 degrees down")]
	#[test_case(90., DirForwardPitch::Up(ForwardPitch::MAX); "90 degrees up")]
	#[test_case(-90., DirForwardPitch::Down(ForwardPitch::MAX); "90 degrees down")]
	fn set_cursor_entity_hit_pitch(angle: f32, forward_pitch: impl Into<Option<DirForwardPitch>>) {
		let mut app = setup();
		let translation = Vec3::new(10., 2., 0.);
		let offset = Quat::from_rotation_x(angle.to_radians()).mul_vec3(Vec3::new(0., 0., -30.));
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_translation(translation + offset))
			.id();
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Cursor)),
				GlobalTransform::from_translation(translation),
				_Animations {
					forward_pitch: None,
				},
			))
			.id();
		app.insert_resource(_RayCast::new().with_mock(|mock| {
			mock.expect_raycast()
				.return_const(Some(MouseHoversOver::Object {
					entity: target,
					point: Vec3::ZERO,
				}));
		}));

		app.update();

		assert_eq_approx!(
			Some(&_Animations {
				forward_pitch: forward_pitch.into()
			}),
			app.world().entity(entity).get::<_Animations>(),
			1e-5,
		);
	}

	#[test]
	fn raycast_excludes_self() {
		let mut app = setup();
		let translation = Vec3::new(10., 2., 0.);
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Cursor)),
				GlobalTransform::from_translation(translation),
				_Animations {
					forward_pitch: None,
				},
			))
			.id();

		app.insert_resource(_RayCast::new().with_mock(|mock| {
			mock.expect_raycast()
				.with(eq(MouseHover::excluding([entity])))
				.once()
				.return_const(None);
		}));

		app.update();
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let translation = Vec3::new(10., 2., 0.);
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Cursor)),
				GlobalTransform::from_translation(translation),
				_Animations {
					forward_pitch: None,
				},
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<_Animations>>(),
		);
	}

	#[test]
	fn act_again_if_changed() {
		let mut app = setup();
		let translation = Vec3::new(10., 2., 0.);
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Cursor)),
				GlobalTransform::from_translation(translation),
				_Animations {
					forward_pitch: None,
				},
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Target>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world().entity(entity).get::<IsChanged<_Animations>>(),
		);
	}
}
