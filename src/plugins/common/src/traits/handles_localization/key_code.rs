use super::Token;
use bevy::prelude::*;

impl From<KeyCode> for Token {
	fn from(value: KeyCode) -> Self {
		let key = format!("{:?}", value);
		Token(format!("key-code-{}", camel_case_to_kebab(key)))
	}
}

fn camel_case_to_kebab(str: String) -> String {
	let mut result = vec![];
	let mut chars = str.chars();

	if let Some(fst) = chars.next() {
		result.extend(fst.to_lowercase());
	}

	for c in chars {
		if c.is_uppercase() {
			result.push('-');
		}
		result.extend(c.to_lowercase());
	}

	result.into_iter().collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test_case(KeyCode::Abort, "key-code-abort"; "abort")]
	#[test_case(KeyCode::KeyA, "key-code-key-a"; "key a")]
	fn key_code_a(key: KeyCode, token: &str) {
		assert_eq!(Token::from(key), Token::from(token));
	}
}
