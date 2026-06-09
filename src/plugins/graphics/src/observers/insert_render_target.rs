use crate::resources::camera_render_target::CameraRenderTarget;
use bevy::{camera::RenderTarget, prelude::*};
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl<T> InsertRenderTarget for T where T: Component {}

pub(crate) trait InsertRenderTarget: Component + Sized {
	fn insert_render_target(
		on_insert: On<Insert, Self>,
		target: Res<CameraRenderTarget<Self>>,
		mut commands: ZyheedaCommands,
	) {
		commands.try_apply_on(&on_insert.entity, |mut e| {
			e.try_insert(RenderTarget::Image(target.handle.clone().into()));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Label;

	fn setup(image: Handle<Image>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(_Label::insert_render_target);
		app.insert_resource(CameraRenderTarget::<_Label>::from(image));

		app
	}

	fn unwrap(target: &RenderTarget) -> Option<(&f32, &Handle<Image>)> {
		let RenderTarget::Image(target) = target else {
			return None;
		};

		Some((&target.scale_factor, &target.handle))
	}

	#[test]
	fn insert_render_target() {
		let handle = new_handle();
		let mut app = setup(handle.clone());

		let entity = app.world_mut().spawn(_Label);

		assert_eq!(
			Some((&1.0, &handle)),
			entity.get::<RenderTarget>().and_then(unwrap),
		);
	}

	#[test]
	fn insert_render_target_when_reinserted() {
		let handle = new_handle();
		let mut app = setup(handle.clone());

		let mut entity = app.world_mut().spawn(_Label);
		entity.remove::<(Camera, RenderTarget)>();
		entity.insert(_Label);

		assert_eq!(
			Some((&1.0, &handle)),
			entity.get::<RenderTarget>().and_then(unwrap),
		);
	}
}
