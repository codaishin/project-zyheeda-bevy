use crate::{components::camera_labels::FirstPass, resources::first_pass_image::FirstPassImage};
use bevy::{prelude::*, render::camera::RenderTarget};
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl FirstPass {
	pub(crate) fn insert_camera(
		trigger: Trigger<OnInsert, Self>,
		first_pass_image: Res<FirstPassImage>,
		mut commands: ZyheedaCommands,
	) {
		commands.try_apply_on(&trigger.target(), |mut e| {
			e.try_insert(Camera {
				hdr: true,
				target: RenderTarget::Image(first_pass_image.handle.clone().into()),
				..default()
			});
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resources::first_pass_image::FirstPassImage;
	use bevy::{math::FloatOrd, render::camera::RenderTarget};
	use testing::{SingleThreadedApp, new_handle};

	fn setup(first_pass_image: Handle<Image>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(FirstPass::insert_camera);
		app.insert_resource(FirstPassImage {
			handle: first_pass_image,
		});

		app
	}

	fn unwrap(target: &RenderTarget) -> Option<(&FloatOrd, &Handle<Image>)> {
		let RenderTarget::Image(target) = target else {
			return None;
		};

		Some((&target.scale_factor, &target.handle))
	}

	#[test]
	fn insert_camera() {
		let handle = new_handle();
		let mut app = setup(handle.clone());

		let entity = app.world_mut().spawn(FirstPass);

		assert_eq!(
			Some((true, Some((&FloatOrd(1.0), &handle)))),
			entity.get::<Camera>().map(|c| (c.hdr, unwrap(&c.target)))
		);
	}

	#[test]
	fn insert_camera_when_reinserted() {
		let handle = new_handle();
		let mut app = setup(handle.clone());

		let mut entity = app.world_mut().spawn(FirstPass);
		entity.remove::<Camera>();
		entity.insert(FirstPass);

		assert_eq!(
			Some((true, Some((&FloatOrd(1.0), &handle)))),
			entity.get::<Camera>().map(|c| (c.hdr, unwrap(&c.target)))
		);
	}
}
