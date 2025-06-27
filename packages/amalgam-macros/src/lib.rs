use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_attribute]
pub fn typeurl(args: TokenStream, input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as syn::Item);
  let args = syn::parse_macro_input!(args as syn::LitStr);
  let type_url = args.value();

  let struct_name = match &input {
    syn::Item::Struct(struct_item) => &struct_item.ident,
    _ => panic!("typeurl macro can only be applied to structs"),
  };

  quote! {
    #input

    impl #struct_name {
      pub const TYPE_URL: &'static str = #type_url;
    }

    impl Into<cosmwasm_std::CosmosMsg> for #struct_name {
      fn into(self) -> cosmwasm_std::CosmosMsg {
        cosmwasm_std::CosmosMsg::Stargate {
          type_url: Self::TYPE_URL.to_string(),
          value: self.encode_to_vec().into(),
        }
      }
    }
  }
  .into()
}
