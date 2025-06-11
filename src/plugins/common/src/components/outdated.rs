use bevy::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Outdated<TComponent>
where
	TComponent: Component,
{
	pub entity: Entity,
	pub component: TComponent,
}
