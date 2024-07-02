use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemTrait, ReturnType};


fn fn_ident_to_enum_ident(ident: &syn::Ident) -> syn::Ident {
    let upper_camel_case = snake_case_to_upper_camel_case(&ident.to_string());
    syn::Ident::new(&upper_camel_case, ident.span())
}


fn snake_case_to_upper_camel_case(snake_case: &str) -> String {
    let mut upper_camel_case = String::new();

    for part in snake_case.split('_') {
        let mut chars = part.chars();
        if let Some(first_char) = chars.next() {
            upper_camel_case.push(first_char.to_ascii_uppercase());
            upper_camel_case.push_str(chars.as_str());
        }
    }
    upper_camel_case
}


fn trait_name_to_struct_ident(ident: &syn::Ident) -> syn::Ident {
        let trait_name = &ident.to_string();
        let trait_suffix = "Trait";
        // Assert that the input ends with the specified suffix
        assert!(trait_name.ends_with(trait_suffix), "Trait name does does not end with the expected {} suffix", trait_suffix);
        // Remove the suffix and return the resulting string slice
        let struct_name = trait_name.strip_suffix(trait_suffix).unwrap().to_string();
        syn::Ident::new(&struct_name, ident.span())
}


#[proc_macro_attribute]
pub fn async_trait_example(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemTrait);
    let trait_name = &input.ident;

    let struct_name = trait_name_to_struct_ident(trait_name);


    // todo change "Messages" and "Responses" to be defined somewhere else

    let message_enum_name =
        syn::Ident::new(&format!("{}Messages", trait_name), trait_name.span());

    let response_enum_name =
        syn::Ident::new(&format!("{}Responses", trait_name), trait_name.span());

    let message_enum_values = input.items.iter().filter_map(|item| {
            if let syn::TraitItem::Method(method) = item {
                let method_name = &method.sig.ident;
                let enum_name = fn_ident_to_enum_ident(method_name);
                let inputs_without_self = method
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|input| match input {
                        syn::FnArg::Receiver(_) => None,
                        syn::FnArg::Typed(pat_type) => Some(pat_type),
                    })
                    .collect::<Vec<_>>();
                Some(quote! {
                    #enum_name{#(#inputs_without_self),*},
                })
            } else {
                None
            }
        });


        let response_enum_values = input.items.iter().filter_map(|item| {
            if let syn::TraitItem::Method(method) = item {
                let method_name = &method.sig.ident;
                let enum_name = fn_ident_to_enum_ident(method_name);
                // let output = &method.sig.output;
                let return_type = match &method.sig.output {
                    ReturnType::Default => {
                        None
                    }
                    ReturnType::Type(_arrow, ty) => {
                        Some(quote! {
                            #ty
                        })
                    }
                };

                Some(quote! {
                    #enum_name (#return_type),
                })
            } else {
                None
            }
        });



        let message_to_invocation_values = input.items.iter().filter_map(|item| {
            if let syn::TraitItem::Method(method) = item {
                let method_name = &method.sig.ident;
                let enum_name = fn_ident_to_enum_ident(method_name);
                let args = method
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|input| {
                        if let FnArg::Typed(pat_type) = input { Some(&pat_type.pat) } else { None }
                    })
                    .collect::<Vec<_>>();

                match &method.sig.output {
                    ReturnType::Default => {
                        eprintln!("no ret val");
                        Some(quote! {
                            #message_enum_name :: #enum_name {#(#args),*} => {
                                self.#method_name(#(#args),*).await;
                                #response_enum_name :: #enum_name ()
                            },
                        })
                    }
                    ReturnType::Type(_arrow, _ty) => {
                        eprintln!("ret val");
                        Some(quote! {
                            #message_enum_name :: #enum_name {#(#args),*} => #response_enum_name :: #enum_name (self.#method_name(#(#args),*).await),
                        })
                    }
                }


            } else {
                None
            }
        });


        let component_client_trait_impl_values = input.items.iter().filter_map(|item| {
            if let syn::TraitItem::Method(method) = item {
                let method_name = &method.sig.ident;
                let enum_name = fn_ident_to_enum_ident(method_name);

                let inputs_without_self = method
                .sig
                .inputs
                .iter()
                .filter_map(|input| match input {
                    syn::FnArg::Receiver(_) => None,
                    syn::FnArg::Typed(pat_type) => Some(pat_type),
                })
                .collect::<Vec<_>>();
    
                let receiver = method.sig.receiver().expect("Receiver not found");
                let receiver = match receiver {
                    syn::FnArg::Receiver(receiver) => Some(receiver),
                    syn::FnArg::Typed(_) => None,
                }
                .expect("Receiver not found");
    
                let output = &method.sig.output;
                let args = method
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|input| {
                        if let FnArg::Typed(pat_type) = input { Some(&pat_type.pat) } else { None }
                    })
                    .collect::<Vec<_>>();
                
                
                match &method.sig.output {
                    ReturnType::Default => {
                        Some(quote! {
                            async fn #method_name(#receiver, #(#inputs_without_self),*) #output {
                                self.send( #message_enum_name :: #enum_name {#(#args),*}).await;
                            }
                        })
                    }
                    ReturnType::Type(_arrow, _ty) => {
                        Some(quote! {
                            async fn #method_name(#receiver, #(#inputs_without_self),*) #output {
                                let res = self.send( #message_enum_name :: #enum_name {#(#args),*}).await;
                                match res {
                                    #response_enum_name :: #enum_name (value) => value,
                                    _ => panic!("Error"),
                                }
                            }
                        })
                    }
                }
                
                
                
                

            } else {
                None
            }
        });




    // let wrapper_struct_name =
    //     syn::Ident::new(&format!("{}ProxyWithRpc", trait_name), trait_name.span());

    // let method_impls = input.items.iter().filter_map(|item| {
    //     if let syn::TraitItem::Method(method) = item {
    //         let method_name = &method.sig.ident;

    //         let inputs_without_self = method
    //             .sig
    //             .inputs
    //             .iter()
    //             .filter_map(|input| match input {
    //                 syn::FnArg::Receiver(_) => None,
    //                 syn::FnArg::Typed(pat_type) => Some(pat_type),
    //             })
    //             .collect::<Vec<_>>();

    //         let receiver = method.sig.receiver().expect("Receiver not found");
    //         let receiver = match receiver {
    //             syn::FnArg::Receiver(receiver) => Some(receiver),
    //             syn::FnArg::Typed(_) => None,
    //         }
    //         .expect("Receiver not found");

    //         let output = &method.sig.output;
    //         let args = method
    //             .sig
    //             .inputs
    //             .iter()
    //             .filter_map(|input| {
    //                 if let FnArg::Typed(pat_type) = input { Some(&pat_type.pat) } else { None }
    //             })
    //             .collect::<Vec<_>>();
    //         Some(quote! {
    //             async fn #method_name(#receiver, #(#inputs_without_self),*) #output {
    //                 let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    //                 let mut inner_clone = self.inner.clone();
    //                 tokio::spawn(async move {
    //                    let result = inner_clone.#method_name(#(#args),*).await;
    //                     tx.send(result).await.expect("Failed to send through channel");
    //                 });

    //                 let stringified_method_name = stringify!(#method_name);



    //                 // let stringified_args = stringify!(#args);

    //                 println!("Sending request to RPC server {}", stringify!(#method_name) );

    //                 rx.recv().await.expect("Failed to receive from channel")
    //             }
    //         })
    //     } else {
    //         None
    //     }
    // });

    let expanded = quote! {
        #input

        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub enum #message_enum_name {
            #(#message_enum_values)*
        }

        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub enum #response_enum_name {
            #(#response_enum_values)*
        }



        #[async_trait]
        impl #trait_name for ComponentClient<#message_enum_name, #response_enum_name> {
             #(#component_client_trait_impl_values)*
        }

        #[async_trait]
        impl ComponentMessageExecutor<#message_enum_name, #response_enum_name> for #struct_name {
            async fn execute(&mut self, message: #message_enum_name) -> #response_enum_name {
                match message {
                    #(#message_to_invocation_values)*
                }
            }
        }
        

        // pub struct #wrapper_struct_name<T: #trait_name + Sync + Send + Clone + 'static> {
        //     inner: T,
        // }

        // #[async_trait::async_trait]
        // impl<T: #trait_name + Sync + Send + Clone + 'static> #trait_name for #wrapper_struct_name<T> {
        //     #(#method_impls)*
        // }

        // impl<T : #trait_name + Sync + Send + Clone + 'static> #wrapper_struct_name<T>  {
        //     pub fn new(value: T) -> Self {
        //         #wrapper_struct_name::<T>{ inner: value}
        //     }
        // }
    };

    TokenStream::from(expanded)
}
