use crate::{components::only_depth_prepass::OnlyDepthPrepass, resources::window_size::WindowSize};
use bevy::{camera::RenderTarget, prelude::*};

impl OnlyDepthPrepass {
	pub(crate) fn update_render_targets(
		window_size: Res<WindowSize>,
		cameras: Query<(&mut RenderTarget, &mut Camera), With<Self>>,
	) {
		let size_changed = window_size.is_changed();

		for (mut target, mut camera) in cameras {
			if !size_changed && !target.is_added() {
				continue;
			}

			// bevy does not process changed `RenderTarget::None` values: https://github.com/bevyengine/bevy/issues/23437,
			// so we set a phony computed to force bevy to process the change
			camera.computed.old_viewport_size = Some(UVec2::default());
			*target = RenderTarget::None {
				size: UVec2 {
					x: window_size.width as u32,
					y: window_size.height as u32,
				},
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resources::window_size::WindowSize;
	use bevy::camera::RenderTarget;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup(size: WindowSize) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(size);
		app.add_systems(
			Update,
			(
				OnlyDepthPrepass::update_render_targets,
				IsChanged::<RenderTarget>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn set_render_target_none_with_proper_size() {
		let size = WindowSize {
			height: 11.,
			width: 111.,
		};
		let mut app = setup(size);
		let entity = app
			.world_mut()
			.spawn((OnlyDepthPrepass, Camera::default()))
			.id();

		app.update();

		assert_eq!(
			format!(
				"{:?}",
				Some(RenderTarget::None {
					size: UVec2 { x: 111, y: 11 }
				})
			),
			format!("{:?}", app.world().entity(entity).get::<RenderTarget>()),
		);
	}

	#[test]
	fn set_old_view_port_value() {
		let size = WindowSize {
			height: 11.,
			width: 111.,
		};
		let mut app = setup(size);
		let entity = app
			.world_mut()
			.spawn((OnlyDepthPrepass, Camera::default()))
			.id();

		app.update();

		assert_eq!(
			Some(Some(UVec2::default())),
			app.world()
				.entity(entity)
				.get::<Camera>()
				.map(|c| c.computed.old_viewport_size),
		);
	}

	#[test]
	fn act_only_once() {
		let size = WindowSize {
			height: 11.,
			width: 111.,
		};
		let mut app = setup(size);
		let entity = app
			.world_mut()
			.spawn((OnlyDepthPrepass, Camera::default()))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<RenderTarget>>(),
		);
	}

	#[test]
	fn act_again_if_window_size_changed() {
		let size = WindowSize {
			height: 11.,
			width: 111.,
		};
		let mut app = setup(size);
		let entity = app
			.world_mut()
			.spawn((OnlyDepthPrepass, Camera::default()))
			.id();

		app.update();
		app.insert_resource(WindowSize {
			height: 22.,
			width: 66.,
		});
		app.update();

		assert_eq!(
			format!(
				"{:?}",
				Some(RenderTarget::None {
					size: UVec2 { x: 66, y: 22 }
				})
			),
			format!("{:?}", app.world().entity(entity).get::<RenderTarget>()),
		);
	}

	#[test]
	fn do_nothing_when_marker_missing() {
		let size = WindowSize {
			height: 11.,
			width: 111.,
		};
		let mut app = setup(size);
		let entity = app
			.world_mut()
			.spawn((RenderTarget::default(), Camera::default()))
			.id();

		app.update();

		assert_eq!(
			format!("{:?}", Some(RenderTarget::default())),
			format!("{:?}", app.world().entity(entity).get::<RenderTarget>()),
		);
	}

	#[test]
	fn set_render_target_when_marker_added_late() {
		let size = WindowSize {
			height: 11.,
			width: 111.,
		};
		let mut app = setup(size);

		app.update();
		let entity = app
			.world_mut()
			.spawn((OnlyDepthPrepass, Camera::default()))
			.id();
		app.update();

		assert_eq!(
			format!(
				"{:?}",
				Some(RenderTarget::None {
					size: UVec2 { x: 111, y: 11 }
				})
			),
			format!("{:?}", app.world().entity(entity).get::<RenderTarget>()),
		);
	}
}
