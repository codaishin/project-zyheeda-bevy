use bevy::{
	color::palettes::{
		css::LIGHT_CYAN,
		tailwind::{CYAN_100, CYAN_200},
	},
	utils::default,
};
use serde::{Deserialize, Serialize};
use shaders::{
	components::material_override::MaterialOverride,
	materials::essence_material::EssenceMaterial,
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum EssenceDto {
	None,
	Force,
}

impl From<EssenceDto> for MaterialOverride {
	fn from(value: EssenceDto) -> Self {
		match value {
			EssenceDto::None => MaterialOverride::None,
			EssenceDto::Force => MaterialOverride::Material(EssenceMaterial {
				texture_color: CYAN_100.into(),
				fill_color: CYAN_200.into(),
				fresnel_color: (LIGHT_CYAN * 1.5).into(),
				..default()
			}),
		}
	}
}
