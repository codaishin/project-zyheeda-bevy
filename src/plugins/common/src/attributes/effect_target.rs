use crate::errors::Unreachable;
use macros::serde_model;
use std::marker::PhantomData;

#[serde_model]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EffectTarget<TEffect> {
	Affected,
	Immune,
	#[serde(skip)]
	_P((PhantomData<TEffect>, Unreachable)),
}
