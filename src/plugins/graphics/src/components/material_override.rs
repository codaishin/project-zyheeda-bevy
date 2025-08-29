use crate::materials::essence_material::EssenceMaterial;
use bevy::{
	color::palettes::{
		css::LIGHT_CYAN,
		tailwind::{CYAN_100, CYAN_200},
	},
	prelude::*,
};
use common::{
	components::essence::Essence,
	traits::register_derived_component::{DerivableFrom, InsertDerivedComponent},
};

#[derive(Component, Debug, PartialEq, Clone, Default)]
#[component(immutable)]
pub enum MaterialOverride {
	#[default]
	None,
	Material(EssenceMaterial),
}

impl<'w, 's> DerivableFrom<'w, 's, Essence> for MaterialOverride {
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::Always;

	type TParam = ();

	fn derive_from(essence: &Essence, _: &()) -> Self {
		match essence {
			Essence::None => MaterialOverride::None,
			Essence::Force => MaterialOverride::Material(EssenceMaterial {
				texture_color: CYAN_100.into(),
				fill_color: CYAN_200.into(),
				fresnel_color: (LIGHT_CYAN * 1.5).into(),
				..default()
			}),
		}
	}
}
