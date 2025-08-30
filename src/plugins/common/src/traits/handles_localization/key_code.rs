use super::Token;
use bevy::prelude::*;

impl From<KeyCode> for Token {
	fn from(value: KeyCode) -> Self {
		let key = format!("{value:?}");
		Token::from(format!("key-code-{}", camel_case_to_kebab(key)))
	}
}

fn camel_case_to_kebab(str: String) -> String {
	let mut result = vec![];
	let mut chars = str.chars();
	let mut last_was_digit = false;

	if let Some(fst) = chars.next() {
		result.extend(fst.to_lowercase());
	}

	for c in chars {
		let is_digit = c.is_ascii_digit();
		let is_new_digit = || is_digit && !last_was_digit;

		if c.is_uppercase() || is_new_digit() {
			result.push('-');
		}

		result.extend(c.to_lowercase());
		last_was_digit = is_digit;
	}

	result.into_iter().collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test_case(KeyCode::Abort, "key-code-abort"; "abort")]
	#[test_case(KeyCode::KeyA, "key-code-key-a"; "key a")]
	#[test_case(KeyCode::Digit3, "key-code-digit-3"; "digit 3")]
	#[test_case(KeyCode::F12, "key-code-f-12"; "F12")]
	fn tokenize(key: KeyCode, token: &str) {
		assert_eq!(Token::from(key), Token::from(token));
	}
}
