use crate::tools::Units;

use super::thread_safe::ThreadSafe;
use bevy::prelude::*;
use std::hash::Hash;

pub trait HandlesMapGeneration
where
	Self::TMap: Component,
	for<'a> Self::TGraph: Graph + From<&'a Self::TMap> + ThreadSafe,
{
	type TMap;
	type TGraph;
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
