use crate::components::renderer::EssenceRender;
use bevy::{
	color::palettes::{
		css::LIGHT_CYAN,
		tailwind::{CYAN_100, CYAN_200},
	},
	utils::default,
};
use serde::{Deserialize, Serialize};
use shaders::materials::essence_material::EssenceMaterial;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum MaterialDto {
	None,
	Force,
}

impl From<MaterialDto> for EssenceRender {
	fn from(value: MaterialDto) -> Self {
		match value {
			MaterialDto::None => EssenceRender::StandardMaterial,
			MaterialDto::Force => EssenceRender::Material(EssenceMaterial {
				texture_color: CYAN_100.into(),
				fill_color: CYAN_200.into(),
				fresnel_color: (LIGHT_CYAN * 1.5).into(),
				..default()
			}),
		}
	}
}
