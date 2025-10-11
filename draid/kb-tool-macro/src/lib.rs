use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{LitInt, LitStr, Token, parse_macro_input};

struct KBMacroInput {
    // Captures the string literal like for the Knowledge Base Name
    name: LitStr,
    // Captures the comma separator
    _comma: Token![,],
    // Captures the integer literal like 10
    num_results: LitInt,
}
// Implementation of the Parse trait to define how the input tokens are consumed.
impl Parse for KBMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(KBMacroInput {
            name: input.parse()?,
            _comma: input.parse()?,
            num_results: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn kb(input: TokenStream) -> TokenStream {
    // 1. Parse the input tokens
    let input = parse_macro_input!(input as KBMacroInput);

    let name_str = input.name.value(); //
    let num_results_lit = input.num_results; // 10 (as LitInt)

    // 2. Generate the dynamic identifier (struct name)
    // format_ident! is crucial for safely constructing identifiers from runtime strings.
    let struct_name = format_ident!("KnowledgeBase{}", name_str);

    // 3. Generate static string literals for use inside the generated code
    //let name_static_str = Literal::string(&format!("knowledge_base_{}", name_str));
    let description_static_str = Literal::string(&format!(
        "Knowledge base containing information on {}",
        name_str
    ));

    // 4. Use quote! to generate the final code block (TokenStream)
    let expanded = quote! {
        {
            // Generated struct definition
            #[derive(Clone)]
            pub struct #struct_name;

            // Generated impl block, replacing placeholders with parsed values
            #[async_trait::async_trait]
            impl Tool for #struct_name {
                fn name(&self) -> &'static str {
                    // Use the generated static string literal
                    #name_str
                }
                fn description(&self) -> &'static str {
                    #description_static_str
                }
                fn parameters(&self) -> Value {
                    json!({
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "Search term to send to knowledge base",
                            },
                        },
                        "required": ["content"],
                    })
                }
                async fn invoke(&self, args: String) -> anyhow::Result<Value> {
                    let client = HttpClient::new();

                    let kb_endpoint = env::var("KNOWLEDGE_BASE_ENDPOINT")
                            .unwrap_or_else(|_e| "http://127.0.0.1:8000".to_string());
                    // The string literal is embedded in the format! call
                    let kb_url = format!("{}/knowledge_base/{}/similar",
                                         kb_endpoint,
                                         #name_str);

                    let args: Value = json::from_str(&args)?;
                    let content = args["content"].as_str().unwrap();

                    // The LitInt for num_results is directly interpolated
                    let body = json!({"text": content, "num_results": #num_results_lit});

                    let response = client.post(kb_url).json(&body).send().await?;
                    let result = response.json::<Vec<String>>().await?;

                    Ok(json!({"result": result}))
                }
            }
            // Return the instance
            Arc::new(#struct_name)
        }
    };

    // 5. Return the resulting TokenStream
    TokenStream::from(expanded)
}
