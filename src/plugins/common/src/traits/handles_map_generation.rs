use super::thread_safe::ThreadSafe;
use crate::{
	tools::Units,
	traits::accessors::get::{GetRef, Getter},
};
use bevy::{ecs::query::QueryFilter, prelude::*};
use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

pub trait HandlesMapGeneration {
	type TMap: Component;
	type TGraph: Graph + for<'a> From<&'a Self::TMap> + ThreadSafe;
	type TSystemSet: SystemSet;

	const SYSTEMS: Self::TSystemSet;

	type TMapRef: Getter<Entity>;

	fn map_mapping_of<TFilter>()
	-> impl IntoSystem<(), EntityMapFiltered<Self::TMapRef, TFilter>, ()>
	where
		TFilter: QueryFilter + 'static;
}

pub trait Graph:
	GraphNode<TNNode = Self::TNode>
	+ GraphSuccessors<TSNode = Self::TNode>
	+ GraphLineOfSight<TLNode = Self::TNode>
	+ GraphObstacle<TONode = Self::TNode>
	+ GraphTranslation<TTNode = Self::TNode>
	+ GraphNaivePath<TNNode = Self::TNode>
{
	type TNode: Eq + Hash + Copy;
}

pub trait GraphNode {
	type TNNode;

	fn node(&self, translation: Vec3) -> Option<Self::TNNode>;
}

pub trait GraphSuccessors {
	type TSNode;

	fn successors(&self, node: &Self::TSNode) -> impl Iterator<Item = Self::TSNode>;
}

pub trait GraphLineOfSight {
	type TLNode;

	fn line_of_sight(&self, a: &Self::TLNode, b: &Self::TLNode) -> bool;
}

pub trait GraphObstacle {
	type TONode;

	fn is_obstacle(&self, node: &Self::TONode) -> bool;
}

pub trait GraphTranslation {
	type TTNode;

	fn translation(&self, node: &Self::TTNode) -> Vec3;
}

pub trait GraphNaivePath {
	type TNNode;

	/// Do some primitive path line of sight computation, to see if a node can be reached from the
	/// given vector. This might be non-computable for non grid based graphs.
	///
	/// Useful when trying to replace path endpoint node translations with real world path start and
	/// end coordinates.
	fn naive_path(&self, origin: Vec3, to: &Self::TNNode, half_width: Units) -> NaivePath;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NaivePath {
	Ok,
	CannotCompute,
	PartialUntil(Vec3),
}

/// Wrapper around a [`HashMap<Entity, TElement>`].
///
/// `TFilter` usage:
/// The provided filter can be used when piping systems to force a consuming system to use
/// matching filters. It can be dropped by converting into a [`HashMap`], if not needed.
pub struct EntityMapFiltered<TElement, TFilter>
where
	TFilter: QueryFilter,
{
	_f: PhantomData<TFilter>,
	map: HashMap<Entity, TElement>,
}

impl<TElement, TFilter> Debug for EntityMapFiltered<TElement, TFilter>
where
	TFilter: QueryFilter,
	TElement: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("EntityMapFiltered")
			.field("_f", &self._f)
			.field("map", &self.map)
			.finish()
	}
}

impl<TElement, TFilter> PartialEq for EntityMapFiltered<TElement, TFilter>
where
	TFilter: QueryFilter,
	TElement: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.map == other.map
	}
}

impl<TElement, TFilter> Default for EntityMapFiltered<TElement, TFilter>
where
	TFilter: QueryFilter,
{
	fn default() -> Self {
		Self {
			_f: PhantomData,
			map: HashMap::default(),
		}
	}
}

impl<T, TElement, TFilter> From<T> for EntityMapFiltered<TElement, TFilter>
where
	TFilter: QueryFilter,
	T: IntoIterator<Item = (Entity, TElement)>,
{
	fn from(iter: T) -> Self {
		Self {
			_f: PhantomData,
			map: HashMap::from_iter(iter),
		}
	}
}
impl<TElement, TFilter> From<EntityMapFiltered<TElement, TFilter>> for HashMap<Entity, TElement>
where
	TFilter: QueryFilter,
{
	fn from(EntityMapFiltered { map, .. }: EntityMapFiltered<TElement, TFilter>) -> Self {
		map
	}
}

impl<TElement, TFilter> GetRef<Entity, TElement> for EntityMapFiltered<TElement, TFilter>
where
	TFilter: QueryFilter,
{
	fn get(&self, key: &Entity) -> Option<&TElement> {
		self.map.get(key)
	}
}
