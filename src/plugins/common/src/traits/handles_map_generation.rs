use super::thread_safe::ThreadSafe;
use crate::{
	tools::Units,
	traits::{
		accessors::get::{GetProperty, Property},
		handles_enemies::EnemyType,
	},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash};

pub trait HandlesMapGeneration {
	type TMap: Component;
	type TGraph: Graph + for<'a> From<&'a Self::TMap> + ThreadSafe;
	type TSystemSet: SystemSet;
	type TMapRef: Component + GetProperty<Entity>;
	type TNewWorldAgent: Component + GetProperty<AgentType>;

	const SYSTEMS: Self::TSystemSet;
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

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum AgentType {
	Player,
	Enemy(EnemyType),
}

impl Property for AgentType {
	type TValue<'a> = Self;
}
