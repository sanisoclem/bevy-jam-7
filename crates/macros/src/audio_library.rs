use proc_macro2::TokenStream;
use quote::quote;
use syn::{
  Data, DeriveInput, Expr, Lit, LitStr, Meta, Token, Variant, parse::Result, punctuated::Punctuated,
};

#[derive(Debug)]
struct AudioInfo {
  variant_name: syn::Ident,
  def: AudioDefInfo,
}

#[derive(Debug)]
enum AudioDefInfo {
  Loop {
    asset_path: String,
  },
  IntroLoop {
    start_path: String,
    main_path: String,
  },
}

struct IntroLoopParsed(Punctuated<LitStr, Token![,]>);
impl syn::parse::Parse for IntroLoopParsed {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    Ok(Self(Punctuated::<LitStr, Token![,]>::parse_terminated(
      input,
    )?))
  }
}

fn extract_def_from_variant(variant: &Variant) -> Result<Option<AudioDefInfo>> {
  for attr in &variant.attrs {
    if attr.path().segments.len() == 1 && attr.path().segments[0].ident == "looped" {
      match &attr.meta {
        Meta::List(meta_list) => {
          let tokens = &meta_list.tokens;
          let expr: Expr = syn::parse2(tokens.clone()).unwrap();
          if let Expr::Lit(expr_lit) = expr
            && let Lit::Str(lit_str) = &expr_lit.lit
          {
            return Ok(Some(AudioDefInfo::Loop {
              asset_path: lit_str.value(),
            }));
          }
        }
        _ => {
          return Err(syn::Error::new_spanned(
            attr,
            "loop attribute must be in the form #[loop(\"asset_path\")]",
          ));
        }
      }
    }
    if attr.path().segments.len() == 1 && attr.path().segments[0].ident == "intro_looped" {
      let args: Vec<_> = attr
        .parse_args::<IntroLoopParsed>()?
        .0
        .into_iter()
        .map(|lit| lit.value())
        .collect();

      if let Some(intro) = args.first()
        && let Some(main) = args.get(1)
      {
        return Ok(Some(AudioDefInfo::IntroLoop {
          start_path: intro.clone(),
          main_path: main.clone(),
        }));
      }
    }
  }
  Ok(None)
}

fn extract_variants_info(data: &Data) -> Result<Vec<AudioInfo>> {
  match data {
    Data::Enum(data_enum) => {
      let mut defs = Vec::new();

      for variant in &data_enum.variants {
        let def = extract_def_from_variant(variant)?.ok_or_else(|| {
          syn::Error::new_spanned(
            &variant.ident,
            "All variants must have a #[loop(\"value\")] or #[intro_loop(\"\",\"\")] attribute",
          )
        })?;

        defs.push(AudioInfo {
          variant_name: variant.ident.clone(),
          def,
        });
      }

      Ok(defs)
    }
    Data::Struct(data_struct) => Err(syn::Error::new_spanned(
      data_struct.struct_token,
      "can only be derived for enums",
    )),
    Data::Union(data_union) => Err(syn::Error::new_spanned(
      data_union.union_token,
      "can only be derived for enums",
    )),
  }
}

pub fn derive_audio_library(ast: DeriveInput) -> Result<TokenStream> {
  let enum_name = &ast.ident;
  let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

  let prefix_infos = extract_variants_info(&ast.data).unwrap();

  let nested_from_arms = prefix_infos.iter().map(|info| {
    let variant_name = &info.variant_name;
    let def = &info.def;

    match def {
      AudioDefInfo::Loop { asset_path } => quote! {
        retval.insert(#enum_name::#variant_name, crate::audio::AudioDef::Looped(asset_server.load(#asset_path)));
      },
      AudioDefInfo::IntroLoop {
        start_path,
        main_path,
      } => quote! {
        retval.insert(#enum_name::#variant_name, crate::audio::AudioDef::IntroLooped { intro: asset_server.load(#start_path),main: asset_server.load(#main_path) });
      },
    }
  });

  Ok(quote! {
      impl #impl_generics crate::audio::AudioLibrary for #enum_name #type_generics #where_clause {
        fn load_all(asset_server: &bevy::asset::AssetServer) -> bevy::platform::collections::HashMap<Self, crate::audio::AudioDef> {
            let mut retval = bevy::platform::collections::HashMap::new();

            #(#nested_from_arms)*
            retval
        }
      }
  })
}
#[cfg(test)]
mod test {
  use quote::quote;
  use syn::DeriveInput;

  #[test]
  fn prefixed_music_library_derive() {
    let input = quote! {
        #[derive(IMusicLibrary, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum GameMusic {
            #[looped("menu.ogg")]
            Menu,
            #[intro_looped("battle_intro.ogg", "battle.ogg")]
            Battle
        }
    };

    let ast = syn::parse2::<DeriveInput>(input).unwrap();

    let out = super::derive_audio_library(ast).unwrap();

    let as_file = syn::parse_file(&out.to_string()).unwrap();

    let formatted = prettyplease::unparse(&as_file);

    insta::assert_snapshot!(formatted);
  }
}
