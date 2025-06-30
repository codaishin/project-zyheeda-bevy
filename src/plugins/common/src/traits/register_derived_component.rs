mod app;

use bevy::prelude::*;

pub trait RegisterDerivedComponent {
	fn register_derived_component<TComponent, TDerived>(&mut self) -> &mut Self
	where
		TComponent: Component,
		for<'a> TDerived: DerivableComponentFrom<TComponent>;
}

pub trait DerivableComponentFrom<TComponent>: for<'a> From<&'a TComponent> + Component {
	const INSERT: InsertDerivedComponent;
}

#[derive(Debug, PartialEq, Default)]
pub enum InsertDerivedComponent {
	#[default]
	IfNew,
	Always,
}
