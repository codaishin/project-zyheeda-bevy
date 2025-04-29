use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyMapDto<TAllKeys, TKeyCode>(pub(crate) HashMap<TAllKeys, TKeyCode>)
where
	TAllKeys: Eq + Hash,
	TKeyCode: PartialEq;

impl<TAllKeys, TKeyCode, const N: usize> From<[(TAllKeys, TKeyCode); N]>
	for KeyMapDto<TAllKeys, TKeyCode>
where
	TAllKeys: Eq + Hash,
	TKeyCode: PartialEq,
{
	fn from(data: [(TAllKeys, TKeyCode); N]) -> Self {
		Self(HashMap::from(data))
	}
}
