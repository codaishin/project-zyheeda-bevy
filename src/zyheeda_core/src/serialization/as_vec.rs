//! Used as a parser for [HashMap]s in conjunction with [serde] (de)serialization.
//!
//! If the map keys can be completely represented as strings, serde can handle the hashmap
//! by itself.
//!
//! Otherwise use this module to treat the map as a [`Vec`] of key value pairs:
//! - only serialization: `#[serde(serialize_with = "as_vec::serialize")]`
//! - only deserialization: `#[serde(deserialize_with = "as_vec::deserialize")]`
//! - both: `#[serde(with = "as_vec")]`
//!
//! # Example
//! ```
//! use zyheeda_core::serialization::as_vec;
//! use serde::{Serialize, Deserialize};
//! use serde_json::{json, to_value, from_value};
//! use std::collections::HashMap;
//!
//! #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
//! struct Key {
//!   id: usize,
//!   name: String,
//! }
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct Container {
//!   #[serde(with = "as_vec")]
//!   map: HashMap<Key, String>
//! }
//!
//! let container = Container {
//!   map: HashMap::from([(
//!     Key { id: 42, name: "my name".to_owned()},
//!     "my value".to_owned())
//!   ])
//! };
//!
//! let value = to_value(&container).unwrap();
//!
//! assert_eq!(
//!   json!({
//!     "map": [
//!       [{ "id": 42, "name": "my name" }, "my value"]
//!     ]
//!   }),
//!   value
//! );
//!
//! let container2 = from_value::<Container>(value).unwrap();
//!
//! assert_eq!(container, container2);
//!
//! ```

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashMap, hash::Hash};

pub fn serialize<S, K, V>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
	K: Serialize,
	V: Serialize,
{
	let vec = map.iter().collect::<Vec<(_, _)>>();
	vec.serialize(serializer)
}

pub fn deserialize<'a, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
	D: Deserializer<'a>,
	K: Deserialize<'a> + Eq + Hash,
	V: Deserialize<'a>,
{
	let vec = Vec::<(K, V)>::deserialize(deserializer)?;
	let mut map = HashMap::with_capacity(vec.len());

	for (i, (key, value)) in vec.into_iter().enumerate() {
		let previous_entry = map.insert(key, value);

		if previous_entry.is_some() {
			return Err(serde::de::Error::custom(format!(
				"duplicate key at index {i}"
			)));
		}
	}

	Ok(map)
}

#[cfg(test)]
mod test_as_vec {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use serde_json::json;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
	enum _Key {
		A(usize),
		B(String),
	}

	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct _Wrapper {
		#[serde(with = "super")]
		map: HashMap<_Key, String>,
	}

	mod serialize {
		use super::*;

		#[test]
		fn as_list_of_lists() {
			let wrapper = _Wrapper {
				map: HashMap::from([
					(_Key::A(42), String::from("A")),
					(_Key::B(String::from("11")), String::from("B")),
				]),
			};

			let value = serde_json::to_value(wrapper).unwrap();

			assert!(
				[
					json!({
						"map":[
							[{ "A": 42 }, "A"],
							[{ "B": "11" }, "B"],
						],
					}),
					json!({
						"map":[
							[{ "B": "11" }, "B"],
							[{ "A": 42 }, "A"],
						],
					}),
				]
				.contains(&value)
			);
		}
	}

	mod deserialize {
		use super::*;

		#[test]
		fn from_list_of_lists() {
			let value = json!({
				"map":[
					[{ "A": 42 }, "A"],
					[{ "B": "11" }, "B"],
				],
			});

			let wrapper = serde_json::from_value::<_Wrapper>(value).unwrap();

			assert_eq!(
				_Wrapper {
					map: HashMap::from([
						(_Key::A(42), String::from("A")),
						(_Key::B(String::from("11")), String::from("B")),
					]),
				},
				wrapper
			);
		}

		#[test]
		fn error_on_duplicate_key() {
			let value = json!({
				"map":[
					[{ "A": 42 }, "A"],
					[{ "A": 42 }, "B"],
				],
			});

			let Err(err) = serde_json::from_value::<_Wrapper>(value) else {
				panic!("EXPECTED ERROR, BUT WAS VALUE");
			};

			assert_eq!("duplicate key at index 1", err.to_string());
		}
	}
}
