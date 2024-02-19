use bevy::{
	asset::{Asset, Handle},
	ecs::system::Resource,
	render::mesh::Mesh,
};
use std::{self, marker::PhantomData};

#[derive(Resource, Default)]
pub struct ModelData<TMaterial: Asset, TModel> {
	pub material: Handle<TMaterial>,
	pub mesh: Handle<Mesh>,
	phantom_data: PhantomData<TModel>,
}

impl<TMaterial: Asset, TModel> ModelData<TMaterial, TModel> {
	pub fn new(material: Handle<TMaterial>, mesh: Handle<Mesh>) -> Self {
		Self {
			material,
			mesh,
			phantom_data: PhantomData,
		}
	}
}

#[derive(Resource)]
pub struct Prefab<TFor, TParent, TChildren> {
	pub parent: TParent,
	pub children: TChildren,
	phantom_data: PhantomData<TFor>,
}

impl<TFor, TParent, TChildren> Prefab<TFor, TParent, TChildren> {
	pub fn new(parent: TParent, children: TChildren) -> Self {
		Self {
			parent,
			children,
			phantom_data: PhantomData,
		}
	}
}
