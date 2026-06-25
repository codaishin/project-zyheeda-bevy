mod render_ui;
mod screen_position;

use crate::components::camera_labels::UiPass;
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::{
	system_params::MarkUnique,
	traits::{
		accessors::get::{GetContextMut, TryApplyOn},
		handles_graphics::CameraHandle,
	},
	zyheeda_commands::ZyheedaCommands,
};

#[derive(SystemParam)]
pub struct UiCameraParamMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	cameras: Query<'w, 's, (Entity, &'static GlobalTransform, &'static Camera), With<UiPass>>,
	_u: MarkUnique<UiCameraParamMut<'w, 's>>,
}

impl GetContextMut<CameraHandle> for UiCameraParamMut<'static, 'static> {
	type TContext<'ctx> = UiCameraContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		_: CameraHandle,
	) -> Self::TContext<'ctx> {
		let commands = &mut param.commands;
		let (entity, transform, camera) = param.cameras.single().unwrap_or_else(|_| {
			(
				commands.spawn(UiPass).id(),
				UiPass::DEFAULT_TRANSFORM,
				UiPass::DEFAULT_CAMERA,
			)
		});

		UiCameraContextMut {
			render_ui: Box::new(move |ui| {
				commands.try_apply_on(&ui, |mut ui| {
					ui.try_insert(UiTargetCamera(entity));
				});
			}),
			transform,
			camera,
		}
	}
}

pub struct UiCameraContextMut<'ctx> {
	render_ui: Box<dyn FnMut(Entity) + 'ctx>,
	camera: &'ctx Camera,
	transform: &'ctx GlobalTransform,
}
