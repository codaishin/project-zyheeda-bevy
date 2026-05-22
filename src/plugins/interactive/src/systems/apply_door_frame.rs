use crate::{
	assets::door_meta::DoorMeta,
	components::{door::ApplyDoorFrame, door_meta_handle::DoorMetaHandle},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, TryApplyOn},
		handles_physics::{
			ConfigureBody,
			NoBodyConfigured,
			TranslationOffsets,
			physical_bodies::{BodyConfig, PhysicsType},
		},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl ApplyDoorFrame {
	pub(crate) fn apply<TBody>(
		doors: Query<(Entity, &DoorMetaHandle), With<Self>>,
		assets: Res<Assets<DoorMeta>>,
		mut commands: ZyheedaCommands,
		mut body: StaticSystemParam<TBody>,
	) where
		TBody: for<'c> GetContextMut<NoBodyConfigured, TContext<'c>: ConfigureBody>,
	{
		for (entity, handle) in doors {
			let Some(meta) = assets.get(handle) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<Self>();
			});

			let key = NoBodyConfigured { entity };
			let Some(mut ctx) = TBody::get_context_mut(&mut body, key) else {
				continue;
			};

			let body = BodyConfig::from_shape(meta.interactive_detection_shape)
				.with_physics_type(PhysicsType::InteractiveFrame);

			ctx.configure_body(body, TranslationOffsets::ZERO);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{assets::door_meta::DoorMeta, components::door_meta_handle::DoorMetaHandle};
	use common::{
		tools::Units,
		traits::handles_physics::{
			TranslationOffsets,
			physical_bodies::{PhysicsType, Shape, ShapeParameters},
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Component, NestedMocks)]
	struct _Body {
		mock: Mock_Body,
	}

	impl Default for _Body {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_configure_body().return_const(());
			})
		}
	}

	#[automock]
	impl ConfigureBody for _Body {
		fn configure_body(&mut self, body: BodyConfig, offsets: TranslationOffsets) {
			self.mock.configure_body(body, offsets);
		}
	}

	fn setup<'a>(doors: impl IntoIterator<Item = (&'a Handle<DoorMeta>, DoorMeta)>) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut door_assets = Assets::default();

		for (id, asset) in doors {
			_ = door_assets.insert(id, asset);
		}

		app.insert_resource(door_assets);
		app.add_systems(Update, ApplyDoorFrame::apply::<Query<&mut _Body>>);

		app
	}

	#[test]
	fn spawn_frame() {
		let handle = new_handle();
		let meta = DoorMeta {
			interactive_detection_shape: ShapeParameters::Sphere {
				radius: Units::from(42.),
			},
			..default()
		};
		let mut app = setup([(&handle, meta)]);
		app.world_mut().spawn((
			ApplyDoorFrame,
			_Body::new().with_mock(assert_config_body),
			DoorMetaHandle(handle),
		));

		app.update();

		fn assert_config_body(mock: &mut Mock_Body) {
			mock.expect_configure_body()
				.once()
				.with(
					eq(BodyConfig {
						physics_type: PhysicsType::InteractiveFrame,
						shape: Shape::Parameters(ShapeParameters::Sphere {
							radius: Units::from(42.),
						}),
						sub_frames: vec![],
					}),
					eq(TranslationOffsets::ZERO),
				)
				.return_const(());
		}
	}

	#[test]
	fn remove_marker() {
		let handle = new_handle();
		let mut app = setup([(&handle, DoorMeta::default())]);
		let entity = app
			.world_mut()
			.spawn((
				ApplyDoorFrame,
				_Body::default(),
				DoorMetaHandle(handle.clone()),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<ApplyDoorFrame>());
	}

	#[test]
	fn remove_marker_when_body_configure_context_missing() {
		let handle = new_handle();
		let mut app = setup([(&handle, DoorMeta::default())]);
		let entity = app
			.world_mut()
			.spawn((ApplyDoorFrame, DoorMetaHandle(handle.clone())))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<ApplyDoorFrame>());
	}

	#[test]
	fn do_not_remove_marker_when_asset_missing() {
		let mut app = setup([]);
		let entity = app
			.world_mut()
			.spawn((
				ApplyDoorFrame,
				_Body::default(),
				DoorMetaHandle(new_handle()),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&ApplyDoorFrame),
			app.world().entity(entity).get::<ApplyDoorFrame>(),
		);
	}

	#[test]
	fn do_nothing_if_marker_missing() {
		let handle = new_handle();
		let meta = DoorMeta {
			interactive_detection_shape: ShapeParameters::Sphere {
				radius: Units::from(42.),
			},
			..default()
		};
		let mut app = setup([(&handle, meta)]);
		app.world_mut().spawn((
			_Body::new().with_mock(asset_configure_not_called),
			DoorMetaHandle(handle),
		));

		app.update();

		fn asset_configure_not_called(mock: &mut Mock_Body) {
			mock.expect_configure_body().never().return_const(());
		}
	}
}
