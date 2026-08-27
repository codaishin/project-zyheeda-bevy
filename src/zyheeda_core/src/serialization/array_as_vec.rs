use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S, I, const N: usize>(array: &[I; N], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
	I: Serialize,
{
	let vec = Vec::from_iter(array);
	vec.serialize(serializer)
}

pub fn deserialize<'a, D, I, const N: usize>(deserializer: D) -> Result<[I; N], D::Error>
where
	D: Deserializer<'a>,
	I: Deserialize<'a>,
{
	let vec = Vec::<I>::deserialize(deserializer)?;

	match <[I; N]>::try_from(vec) {
		Ok(v) => Ok(v),
		Err(v) if v.len() > N => Err(serde::de::Error::custom(format!(
			"source array with length {} exceeds target array with length {}",
			v.len(),
			N,
		))),
		Err(v) => Err(serde::de::Error::custom(format!(
			"target array with length {} exceeds source array with length {}",
			N,
			v.len(),
		))),
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use serde_json::json;

	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct _Data<const N: usize>(#[serde(with = "super")] [u32; N]);

	#[test]
	fn serialize() {
		let d = _Data([1, 2, 3]);

		let v = serde_json::to_value(d).unwrap();

		assert_eq!(json!([1, 2, 3]), v);
	}

	#[test]
	fn deserialize() {
		let v = json!([1, 2, 3]);

		let d = serde_json::from_value::<_Data<3>>(v).unwrap();

		assert_eq!(_Data([1, 2, 3]), d);
	}

	#[test]
	fn deserialize_too_many_items() {
		let v = json!([1, 2, 3]);

		let Err(e) = serde_json::from_value::<_Data<2>>(v) else {
			panic!("EXPECTED ERROR, BUT WAS VALUE");
		};

		assert_eq!(
			"source array with length 3 exceeds target array with length 2",
			e.to_string()
		);
	}

	#[test]
	fn deserialize_too_few_items() {
		let v = json!([1, 2, 3]);

		let Err(e) = serde_json::from_value::<_Data<4>>(v) else {
			panic!("EXPECTED ERROR, BUT WAS VALUE");
		};

		assert_eq!(
			"target array with length 4 exceeds source array with length 3",
			e.to_string()
		);
	}
}
