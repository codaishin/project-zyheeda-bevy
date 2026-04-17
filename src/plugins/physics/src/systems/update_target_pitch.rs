use crate::components::{center_offset::CenterOffset, target::Target};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::{Get, GetContextMut},
		handles_animations::{Animations, DirForwardPitch, ForwardPitch, GetForwardPitchMut},
		handles_physics::{MouseHover, MouseHoversOver, Raycast},
		handles_skill_physics::{Cursor, SkillTarget},
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::f32::consts::FRAC_PI_2;

impl Target {
	pub(crate) fn update_pitch<TRayCast, TAnimations>(
		mut animations: StaticSystemParam<TAnimations>,
		mut ray_caster: StaticSystemParam<TRayCast>,
		targets: Query<(Entity, &Self, &GlobalTransform, Option<&CenterOffset>)>,
		transforms: Query<(&GlobalTransform, Option<&CenterOffset>)>,
		commands: ZyheedaCommands,
	) where
		for<'w, 's> TRayCast: SystemParam<Item<'w, 's>: Raycast<MouseHover>>,
		for<'c> TAnimations:
			SystemParam + GetContextMut<Animations, TContext<'c>: GetForwardPitchMut>,
	{
		for (entity, target, transform, offset) in targets {
			let key = Animations { entity };
			let Some(mut ctx) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};
			let pitch = target.get_pitch(
				entity,
				transform,
				offset,
				transforms,
				&commands,
				&mut ray_caster,
			);

			*ctx.get_forward_pitch_mut() = pitch;
		}
	}

	fn get_pitch(
		&self,
		entity: Entity,
		transform: &GlobalTransform,
		offset: Option<&CenterOffset>,
		transforms: Query<(&GlobalTransform, Option<&CenterOffset>)>,
		commands: &ZyheedaCommands,
		ray_cast: &mut impl Raycast<MouseHover>,
	) -> Option<DirForwardPitch> {
		match self.0.as_ref()? {
			SkillTarget::Entity(entity) => {
				let target = commands.get(entity)?;
				let (target_transform, target_offset) = transforms.get(target).ok()?;
				let target = with_offset(target_transform, target_offset);
				get_pitch(transform, offset, target)
			}
			SkillTarget::Cursor(Cursor::TerrainHover) => {
				match ray_cast.raycast(MouseHover::excluding([entity]))? {
					MouseHoversOver::Point(point) => get_pitch(transform, offset, point),
					MouseHoversOver::Object { entity, .. } => {
						let (target_transform, target_offset) = transforms.get(entity).ok()?;
						let target = with_offset(target_transform, target_offset);
						get_pitch(transform, offset, target)
					}
				}
			}
			SkillTarget::Cursor(Cursor::Direction) => None,
		}
	}
}

fn get_pitch(
	transform: &GlobalTransform,
	offset: Option<&CenterOffset>,
	to: Vec3,
) -> Option<DirForwardPitch> {
	let dir = (to - with_offset(transform, offset)).try_normalize()?;
	let pitch = ForwardPitch::try_from((dir.y.asin() / FRAC_PI_2).abs()).ok()?;

	if dir.y > 0. {
		Some(DirForwardPitch::Up(pitch))
	} else {
		Some(DirForwardPitch::Down(pitch))
	}
}

fn with_offset(transform: &GlobalTransform, offset: Option<&CenterOffset>) -> Vec3 {
	match offset {
		Some(CenterOffset(offset)) => transform.translation() + Vec3::new(0., *offset, 0.),
		None => transform.translation(),
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
			handles_skill_physics::{Cursor, SkillTarget},
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
	fn set_target_entity_pitch_with_offset(
		angle: f32,
		forward_pitch: impl Into<Option<DirForwardPitch>>,
	) {
		let mut app = setup();
		let translation = Vec3::new(10., 2., 0.);
		let offset = Quat::from_rotation_x(angle.to_radians()).mul_vec3(Vec3::new(0., 0., -30.));
		let target_entity = PersistentEntity::default();
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Entity(target_entity))),
				CenterOffset(3.),
				GlobalTransform::from_translation(translation),
				_Animations {
					forward_pitch: None,
				},
			))
			.id();

		app.world_mut().spawn((
			target_entity,
			GlobalTransform::from_translation(translation + Vec3::new(0., 3., 0.) + offset),
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
	fn set_target_entity_pitch_with_target_offset(
		angle: f32,
		forward_pitch: impl Into<Option<DirForwardPitch>>,
	) {
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
			GlobalTransform::from_translation(translation + Vec3::new(0., -3., 0.) + offset),
			CenterOffset(3.),
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
				Target(Some(SkillTarget::Cursor(Cursor::TerrainHover))),
				GlobalTransform::from_translation(translation),
				_Animations {
					forward_pitch: None,
				},
			))
			.id();
		app.insert_resource(_RayCast::new().with_mock(|mock| {
			mock.expect_raycast()
				.return_const(Some(MouseHoversOver::Point(translation + offset)));
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
	fn set_cursor_terrain_hit_pitch_with_offset(
		angle: f32,
		forward_pitch: impl Into<Option<DirForwardPitch>>,
	) {
		let mut app = setup();
		let translation = Vec3::new(10., 2., 0.);
		let offset = Quat::from_rotation_x(angle.to_radians()).mul_vec3(Vec3::new(0., 0., -30.));
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Cursor(Cursor::TerrainHover))),
				GlobalTransform::from_translation(translation),
				CenterOffset(3.),
				_Animations {
					forward_pitch: None,
				},
			))
			.id();
		app.insert_resource(_RayCast::new().with_mock(|mock| {
			mock.expect_raycast()
				.return_const(Some(MouseHoversOver::Point(
					translation + Vec3::new(0., 3., 0.) + offset,
				)));
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
				Target(Some(SkillTarget::Cursor(Cursor::TerrainHover))),
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

	#[test_case(0., None; "0 degrees")]
	#[test_case(45., ForwardPitch::try_from(0.5).ok().map(DirForwardPitch::Up); "45 degrees up")]
	#[test_case(-45., ForwardPitch::try_from(0.5).ok().map(DirForwardPitch::Down); "45 degrees down")]
	#[test_case(90., DirForwardPitch::Up(ForwardPitch::MAX); "90 degrees up")]
	#[test_case(-90., DirForwardPitch::Down(ForwardPitch::MAX); "90 degrees down")]
	fn set_cursor_entity_hit_pitch_with_offset(
		angle: f32,
		forward_pitch: impl Into<Option<DirForwardPitch>>,
	) {
		let mut app = setup();
		let translation = Vec3::new(10., 2., 0.);
		let offset = Quat::from_rotation_x(angle.to_radians()).mul_vec3(Vec3::new(0., 0., -30.));
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_translation(
				translation + Vec3::new(0., 3., 0.) + offset,
			))
			.id();
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Cursor(Cursor::TerrainHover))),
				GlobalTransform::from_translation(translation),
				CenterOffset(3.),
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

	#[test_case(0., None; "0 degrees")]
	#[test_case(45., ForwardPitch::try_from(0.5).ok().map(DirForwardPitch::Up); "45 degrees up")]
	#[test_case(-45., ForwardPitch::try_from(0.5).ok().map(DirForwardPitch::Down); "45 degrees down")]
	#[test_case(90., DirForwardPitch::Up(ForwardPitch::MAX); "90 degrees up")]
	#[test_case(-90., DirForwardPitch::Down(ForwardPitch::MAX); "90 degrees down")]
	fn set_cursor_entity_hit_pitch_with_hit_offset(
		angle: f32,
		forward_pitch: impl Into<Option<DirForwardPitch>>,
	) {
		let mut app = setup();
		let translation = Vec3::new(10., 2., 0.);
		let offset = Quat::from_rotation_x(angle.to_radians()).mul_vec3(Vec3::new(0., 0., -30.));
		let target = app
			.world_mut()
			.spawn((
				GlobalTransform::from_translation(translation + Vec3::new(0., -3., 0.) + offset),
				CenterOffset(3.),
			))
			.id();
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Cursor(Cursor::TerrainHover))),
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
	fn do_not_set_pitch_for_directional_cursor() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Cursor(Cursor::Direction))),
				GlobalTransform::default(),
				_Animations {
					forward_pitch: Some(DirForwardPitch::Up(ForwardPitch::MAX)),
				},
			))
			.id();
		app.insert_resource(_RayCast::new().with_mock(|mock| {
			mock.expect_raycast()
				.return_const(Some(MouseHoversOver::Point(Vec3::new(1., 2., 3.))));
		}));

		app.update();

		assert_eq_approx!(
			Some(&_Animations {
				forward_pitch: None
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
				Target(Some(SkillTarget::Cursor(Cursor::TerrainHover))),
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
}
