use super::thread_safe::ThreadSafe;
use crate::{
	tools::Units,
	traits::{
		accessors::get::{GetContextMut, GetProperty, Property},
		handles_enemies::EnemyType,
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};
use std::{
	fmt::Debug,
	hash::Hash,
	ops::{Deref, DerefMut},
};

pub trait HandlesMapGeneration {
	const SYSTEMS: Self::TSystemSet;
	type TSystemSet: SystemSet;

	type TNewMapAgent<'w, 's>: SystemParam
		+ for<'c> GetContextMut<AgentPrefab, TContext<'c>: SetMapAgentPrefab>;

	type TGraph: Graph + for<'a> From<&'a Self::TMap> + ThreadSafe;

	type TMap: Component;
	type TMapRef: Component + GetProperty<Entity>;
}

pub type NewMapAgentParamMut<'w, 's, TMaps> = <TMaps as HandlesMapGeneration>::TNewMapAgent<'w, 's>;

pub trait Graph:
	GraphNode<TNNode = Self::TNode>
	+ GraphSuccessors<TSNode = Self::TNode>
	+ GraphLineOfSight<TLNode = Self::TNode>
	+ GraphObstacle<TONode = Self::TNode>
	+ GraphGroundPosition<TTNode = Self::TNode>
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

pub trait GraphGroundPosition {
	type TTNode;

	fn ground_position(&self, node: &Self::TTNode) -> GroundPosition;
}

pub trait GraphNaivePath {
	type TNNode;

	/// Do some primitive path line of sight computation, to see if a node can be reached from the
	/// given vector. This might be non-computable for non grid based graphs.
	///
	/// Useful when trying to replace path endpoint node translations with real world path start and
	/// end coordinates.
	fn naive_path(&self, translation: Vec3, to: &Self::TNNode, half_width: Units) -> NaivePath;
}

#[derive(Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize)]
pub struct GroundPosition(pub Vec3);

impl GroundPosition {
	pub const ZERO: Self = Self(Vec3::ZERO);
	pub const ONE: Self = Self(Vec3::ONE);
}

impl From<Vec3> for GroundPosition {
	fn from(ground_position: Vec3) -> Self {
		Self(ground_position)
	}
}

impl Deref for GroundPosition {
	type Target = Vec3;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for GroundPosition {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub trait SetMapAgentPrefab {
	fn set_map_agent_prefab(
		&mut self,
		prefab: fn(ZyheedaEntityCommands, GroundPosition, AgentType),
	);
}

pub struct AgentPrefab;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NaivePath {
	Ok,
	CannotCompute,
	PartialUntil(GroundPosition),
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum AgentType {
	Player,
	Enemy(EnemyType),
}

impl Property for AgentType {
	type TValue<'a> = Self;
}
