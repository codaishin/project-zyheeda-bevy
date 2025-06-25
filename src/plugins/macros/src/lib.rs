use proc_macro::TokenStream;
use quote::quote;
use syn::{
	Data,
	DeriveInput,
	Error,
	Fields,
	Ident,
	Path,
	Type,
	parse_macro_input,
	spanned::Spanned,
};

#[proc_macro_derive(ClampZeroPositive)]
pub fn clamp_zero_positive_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let ident = input.ident;
	let implementation = quote! {
		impl ClampZeroPositive for #ident {
			fn new(value: f32) -> Self {
				if value < 0. {
					Self(0.)
				} else {
					Self(value)
				}
			}
		}

		impl Default for #ident {
			fn default() -> Self {
				Self(0.)
			}
		}

		impl Deref for #ident {
			type Target = f32;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}
	};

	implementation.into()
}

#[proc_macro_derive(NestedMocks)]
pub fn nested_mocks_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let fields = match get_fields(&input) {
		Ok(fields) => fields,
		Err(error) => return error.into(),
	};

	let ident = &input.ident;
	let mut inits = Vec::new();
	let mut traits = Vec::new();

	for (field_name, ty) in fields {
		inits.push(quote! {
			#field_name: #ty::default(),
		});
		traits.push(quote! {
			impl common::traits::nested_mock::NestedMocks<#ty> for #ident {
				fn with_mock(mut self, mut configure_fn: impl FnMut(&mut #ty)) -> Self {
					configure_fn(&mut self.#field_name);
					self
				}
			}
		})
	}

	let implementation = quote! {
		impl #ident {
			pub fn new() -> Self {
				Self { #(#inits)* }
			}

		}

		#(#traits)*
	};

	TokenStream::from(implementation)
}

enum SetupMockCompileError {
	NotAStruct,
	NoNamedFields,
}

impl From<SetupMockCompileError> for TokenStream {
	fn from(value: SetupMockCompileError) -> Self {
		match value {
			SetupMockCompileError::NotAStruct => TokenStream::from(quote! {
				compile_error!("SetupMocks can only be derived for structs");
			}),
			SetupMockCompileError::NoNamedFields => TokenStream::from(quote! {
				compile_error!("SetupMocks can only be derived for structs with named fields");
			}),
		}
	}
}

fn get_fields(
	input: &DeriveInput,
) -> Result<impl Iterator<Item = (Option<&Ident>, &Type)>, SetupMockCompileError> {
	let Data::Struct(data_struct) = &input.data else {
		return Err(SetupMockCompileError::NotAStruct);
	};

	let Fields::Named(ref fields) = data_struct.fields else {
		return Err(SetupMockCompileError::NoNamedFields);
	};

	Ok(fields
		.named
		.iter()
		.map(|field| (field.ident.as_ref(), &field.ty)))
}

#[proc_macro]
pub fn skill_asset(input: TokenStream) -> TokenStream {
	let Ok(syn::Lit::Str(literal)) = syn::parse::<syn::Lit>(input.clone()) else {
		return TokenStream::from(quote! {
			compile_error!("Only string literals are accepted")
		});
	};

	let asset_path = format!("skills/{}.skill", literal.value());
	let path = format!("assets/{}", asset_path);

	if !std::path::Path::new(&path).exists() {
		return TokenStream::from(quote! {
			compile_error!("No skill with that name found in `assets/skills/`")
		});
	}

	TokenStream::from(quote! {
		#asset_path
	})
}

#[proc_macro]
pub fn item_asset(input: TokenStream) -> TokenStream {
	let Ok(syn::Lit::Str(literal)) = syn::parse::<syn::Lit>(input.clone()) else {
		return TokenStream::from(quote! {
			compile_error!("Only string literals are accepted")
		});
	};

	let asset_path = format!("items/{}.item", literal.value());
	let path = format!("assets/{}", asset_path);

	if !std::path::Path::new(&path).exists() {
		return TokenStream::from(quote! {
			compile_error!("No item with that name found in `assets/items/`")
		});
	}

	TokenStream::from(quote! {
		#asset_path
	})
}

/// Implements the `SavableComponent` trait.
///
/// This derive macro supports the following optional attribute:
///
/// `#[savable_component(...)]`
/// - `dto = Type` *(optional)*:
///   Sets `SavableComponent::TDto` to the given type. Defaults to `Self`.
///
/// - `has_priority` *(optional, flag)*:
///   When present, `SavableComponent::PRIORITY` will be `true`. Defaults to `false`.
#[proc_macro_derive(SavableComponent, attributes(savable_component))]
pub fn derive_savable_component(input: TokenStream) -> TokenStream {
	let common = match crate_root("common") {
		Ok(common) => common,
		Err(error) => return error,
	};
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (impl_generics, type_generics, where_clause) = &input.generics.split_for_impl();
	let mut dto = None;
	let mut priority = false;

	for attr in input.attrs.iter() {
		if !attr.path().is_ident("savable_component") {
			continue;
		}

		let result = attr.parse_nested_meta(|nested| match nested.path.get_ident() {
			Some(ident) if ident == "dto" => {
				dto = Some(nested.value()?.parse::<Type>()?);
				Ok(())
			}
			Some(ident) if ident == "has_priority" => {
				priority = true;
				Ok(())
			}
			Some(other) => Err(Error::new(
				nested.path.span(),
				format!("unknown key word '{other}'"),
			)),
			None => Ok(()),
		});

		if let Err(error) = result {
			return TokenStream::from(error.to_compile_error());
		}
	}

	let dto = match dto {
		Some(dto) => quote! {#dto},
		None => quote! {Self},
	};

	TokenStream::from(quote! {
		impl #impl_generics #common::traits::handles_saving::SavableComponent for #name #type_generics #where_clause {
			type TDto = #dto;
			const PRIORITY: bool = #priority;
		}
	})
}

fn crate_root(name: &str) -> Result<Path, TokenStream> {
	let name = match std::env::var("CARGO_PKG_NAME") {
		Ok(n) if n.as_str() == name => "crate",
		_ => name,
	};

	let name = match name.parse::<TokenStream>() {
		Ok(name) => name,
		Err(error) => {
			let error = format!("{name}: {error}");
			return Err(TokenStream::from(quote! {compile_error!(#error)}));
		}
	};

	match syn::parse(name) {
		Ok(name) => Ok(name),
		Err(error) => Err(TokenStream::from(error.to_compile_error())),
	}
}
