mod app;

use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};

pub trait RegisterDerivedComponent {
	fn register_derived_component<TComponent, TDerived>(&mut self) -> &mut Self
	where
		TComponent: Component,
		for<'w, 's> TDerived: DerivableFrom<'w, 's, TComponent>;
}

pub trait DerivableFrom<'w, 's, TComponent>: Component + Sized {
	const INSERT: InsertDerivedComponent;

	type TParam: SystemParam;

	fn derive_from(
		entity: Entity,
		component: &TComponent,
		param: &SystemParamItem<Self::TParam>,
	) -> Option<Self>;
}

#[derive(Debug, PartialEq, Default)]
pub enum InsertDerivedComponent {
	#[default]
	IfNew,
	Always,
}
