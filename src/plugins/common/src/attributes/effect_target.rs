pub mod force;
pub mod gravity;

use crate::errors::Unreachable;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum EffectTarget<TEffect> {
	Affected,
	Immune,
	#[serde(skip)]
	_P((PhantomData<TEffect>, Unreachable)),
}
