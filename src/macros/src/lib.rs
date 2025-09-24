use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
	Data,
	DeriveInput,
	Error,
	Expr,
	Field,
	Fields,
	Ident,
	ItemImpl,
	Lit,
	Path,
	Token,
	Type,
	braced,
	parse::{Parse, ParseStream},
	parse_macro_input,
	spanned::Spanned,
};

#[proc_macro_derive(ClampZeroPositive)]
pub fn clamp_zero_positive_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let ident = input.ident;
	let implementation = quote! {
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

		impl From<f32> for #ident {
			fn from(value: f32) -> Self {
				if value > 0. {
					Self(value)
				} else {
					Self(0.)
				}
			}
		}

		impl<'de> serde::Deserialize<'de> for #ident {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				let v = f32::deserialize(deserializer)?;
				Ok(Self::from(v))
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
			impl testing::NestedMocks<#ty> for #ident {
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
	let path = format!("assets/{asset_path}");

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
	let path = format!("assets/{asset_path}");

	if !std::path::Path::new(&path).exists() {
		return TokenStream::from(quote! {
			compile_error!("No item with that name found in `assets/items/`")
		});
	}

	TokenStream::from(quote! {
		#asset_path
	})
}

#[proc_macro]
pub fn agent_asset(input: TokenStream) -> TokenStream {
	let Ok(syn::Lit::Str(literal)) = syn::parse::<syn::Lit>(input.clone()) else {
		return TokenStream::from(quote! {
			compile_error!("Only string literals are accepted")
		});
	};

	let asset_path = format!("agents/{}.agent", literal.value());
	let path = format!("assets/{asset_path}");

	if !std::path::Path::new(&path).exists() {
		return TokenStream::from(quote! {
			compile_error!("No agent with that name found in `assets/agents/`")
		});
	}

	TokenStream::from(quote! {
		#asset_path
	})
}

struct MinArgs {
	ty: Type,
	lit: Lit,
}

impl Parse for MinArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let ty = input.parse::<Type>()?;
		_ = input.parse::<Token![,]>()?;
		let lit = input.parse::<Lit>()?;
		_ = input.parse::<Option<Token![,]>>()?;
		Ok(Self { ty, lit })
	}
}

/// Uses const evaluation against a `#ty::try_new(#lit) -> Result<#ty, SomeError>` function.
///
/// Prevents compilation if `#ty::try_new(#lit)` returns an error.
///
/// Arguments:
/// - `#ty:ty`: the type containing `try_new()`
/// - `#lit:literal`: a literal from which to instantiate `#ty`
#[proc_macro]
pub fn new_valid(input: TokenStream) -> TokenStream {
	let MinArgs { ty, lit } = parse_macro_input!(input);

	TokenStream::from(quote! {{
		const new: #ty = match <#ty>::try_new(#lit) {
			Ok(n) => n,
			Err(_) => panic!(concat!(stringify!(#ty), ": ", #lit, " is invalid"))
		};
		new
	}})
}

struct Low {
	excl: Option<Token![>]>,
	expr: Expr,
}

impl Parse for Low {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Self {
			excl: input.parse()?,
			expr: input.parse()?,
		})
	}
}

struct High {
	excl: Option<Token![<]>,
	expr: Expr,
}

impl Parse for High {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Self {
			excl: input.parse()?,
			expr: input.parse()?,
		})
	}
}

/// Derive in-range limits for a struct with one unnamed field.
///
/// By default the `low` and `high` limits are inclusive.
/// - `low` can be made exclusive by prefixing `>` to the value
/// - `high` can be made exclusive by prefixing `<` to the value
///
/// It is recommended to mark the unnamed field private in order to prevent bypassing the
/// limit check.
/// ```
#[proc_macro_derive(InRange, attributes(in_range))]
pub fn derive_in_range(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let core = match crate_root("zyheeda_core") {
		Ok(core) => core,
		Err(error) => return error,
	};
	let name = &input.ident;
	let (impl_generics, type_generics, where_clause) = &input.generics.split_for_impl();

	let Some([field]) = get_unnamed_fields(&input) else {
		return TokenStream::from(
			Error::new(
				name.span(),
				"InBetween: must be a struct with one unnamed field",
			)
			.to_compile_error(),
		);
	};
	let ty = &field.ty;

	let Some(in_range) = find_attribute(&input, "in_range") else {
		return TokenStream::from(
			Error::new(name.span(), "InBetween: missing attribute `in_range`").to_compile_error(),
		);
	};
	let mut low = None;
	let mut high = None;
	let result = in_range.parse_nested_meta(|nested| match nested.path.get_ident() {
		Some(ident) if ident == "low" => {
			low = Some(nested.value()?.parse::<Low>()?);
			Ok(())
		}
		Some(ident) if ident == "high" => {
			high = Some(nested.value()?.parse::<High>()?);
			Ok(())
		}
		Some(other) => Err(Error::new(
			nested.path.span(),
			format!("InBetween: unknown key word '{other}'"),
		)),
		None => Ok(()),
	});

	if let Err(error) = result {
		return TokenStream::from(error.to_compile_error());
	}

	let Some(low) = low else {
		return TokenStream::from(
			Error::new(name.span(), "InBetween: missing `in_range` low").to_compile_error(),
		);
	};
	let low_fns = match low.excl.is_some() {
		true => quote! {
			const fn low_ok(value: #ty) -> bool {
				Self::LIMITS.0 < value
			}

			const fn low_limit(value: #ty) -> #core::errors::Limit<#ty> {
				#core::errors::Limit::Exclusive(value)
			}
		},
		false => quote! {
			const fn low_ok(value: #ty) -> bool {
				Self::LIMITS.0 <= value
			}

			const fn low_limit(value: #ty) -> #core::errors::Limit<#ty> {
				#core::errors::Limit::Inclusive(value)
			}
		},
	};
	let low = low.expr;

	let Some(high) = high else {
		return TokenStream::from(
			Error::new(name.span(), "InBetween: missing `in_range` high").to_compile_error(),
		);
	};
	let high_fns = match high.excl.is_some() {
		true => quote! {
			const fn high_ok(value: #ty) -> bool {
				Self::LIMITS.1 > value
			}

			const fn high_limit(value: #ty) -> #core::errors::Limit<#ty> {
				#core::errors::Limit::Exclusive(value)
			}
		},
		false => quote! {
			const fn high_ok(value: #ty) -> bool {
				Self::LIMITS.1 >= value
			}

			const fn high_limit(value: #ty) -> #core::errors::Limit<#ty> {
				#core::errors::Limit::Inclusive(value)
			}
		},
	};
	let high = high.expr;

	TokenStream::from(quote! {
		impl #impl_generics #name #type_generics #where_clause {
			const LIMITS: (#ty, #ty) = match (#low, #high) {
				(l, h) if l < h => (l, h),
				_ => panic!("`InBetween: low` must be lesser than `high`")
			};

			#low_fns

			#high_fns

			pub const fn try_new(value: #ty) -> Result<Self, #core::errors::NotInRange<#ty>> {
				if !Self::low_ok(value) || !Self::high_ok(value) {
					return Err(#core::errors::NotInRange {
						lower_limit: Self::low_limit(Self::LIMITS.0),
						upper_limit: Self::high_limit(Self::LIMITS.1),
						value,
					});
				}

				Ok(Self(value))
			}

			pub fn unwrap(self) -> #ty {
				self.0
			}
		}

		impl #impl_generics TryFrom<#ty> for #name #type_generics #where_clause {
			type Error = #core::errors::NotInRange<#ty>;

			fn try_from(value: #ty) -> Result<Self, Self::Error> {
				Self::try_new(value)
			}
		}

		impl #impl_generics std::ops::Deref for #name #type_generics #where_clause {
			type Target = #ty;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}
	})
}

fn find_attribute<'a>(input: &'a DeriveInput, name: &str) -> Option<&'a syn::Attribute> {
	input.attrs.iter().find(|attr| attr.path().is_ident(name))
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
	let mut where_clause = match where_clause.cloned() {
		Some(where_clause) => where_clause,
		None => syn::WhereClause {
			where_token: Default::default(),
			predicates: syn::punctuated::Punctuated::new(),
		},
	};

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

	where_clause.predicates.push(syn::parse_quote! {
		Self: bevy::prelude::Component +
			Sized +
			Clone +
			#common::traits::handles_custom_assets::TryLoadFrom<
				#dto,
				TInstantiationError = #common::errors::Unreachable
			>
	});
	where_clause.predicates.push(syn::parse_quote! {
		#dto: From<Self> + serde::Serialize + serde::de::DeserializeOwned
	});

	TokenStream::from(quote! {
		impl #impl_generics #common::traits::handles_saving::SavableComponent for #name #type_generics #where_clause {
			type TDto = #dto;
			const PRIORITY: bool = #priority;
		}
	})
}

fn get_unnamed_fields<const N: usize>(input: &DeriveInput) -> Option<[&Field; N]> {
	let data_struct = match &input.data {
		Data::Struct(data_struct) => data_struct,
		_ => return None,
	};

	let unnamed_fields = match &data_struct.fields {
		Fields::Unnamed(fields) => &fields.unnamed,
		_ => return None,
	};

	unnamed_fields.iter().collect::<Vec<_>>().try_into().ok()
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

struct MockInput {
	ident: Ident,
	_brace: syn::token::Brace,
	implementations: Vec<ItemImpl>,
}

impl Parse for MockInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let ident: Ident = input.parse()?;
		let content;
		let _brace = braced!(content in input);

		if !content.is_empty() {
			return Err(content.error("expected empty {} after type name"));
		}

		let mut implementations = Vec::new();
		while !input.is_empty() {
			implementations.push(input.parse()?);
		}

		Ok(MockInput {
			ident,
			_brace,
			implementations,
		})
	}
}

/// Wrapper around `mockall::mock` that adds initialization logic
///
/// # Example
/// ```ignore
/// simple_mock! {
///   _T {}
///   impl SomeTrait for _T {
///     fn some_func(&self);
///   }
/// }
///
/// let t = Mock_T::new_mock(|mock| {
///   mock
///     .expect_some_func()
///     .times(1)
///     .return_const(());
/// });
///
/// t.some_func();
/// ```
#[proc_macro]
pub fn simple_mock(tokens: TokenStream) -> TokenStream {
	let MockInput {
		ident,
		implementations,
		..
	} = parse_macro_input!(tokens as MockInput);
	let mock_ident = format_ident!("Mock{ident}");

	TokenStream::from(quote! {
		mockall::mock! {
			#ident {}
			#(#implementations)*
		}

		impl testing::Mock for #mock_ident {
			fn new_mock(mut configure: impl FnMut(&mut Self)) -> Self {
				let mut mock = Self::default();
				configure(&mut mock);
				mock
			}
		}
	})
}
