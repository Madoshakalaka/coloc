use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token};

const NUM_LANGS: usize = 3;
const EN: usize = 0;
const JA: usize = 1;
const ZH: usize = 2;

#[cfg(all(feature = "lang-ja", feature = "lang-zh-hans"))]
compile_error!("Cannot enable both `lang-ja` and `lang-zh-hans`");

#[derive(Clone)]
enum Item {
    Text(String),
    Link { text: String, href: String },
    Placeholder,
}

struct ColocInput {
    block_name: syn::Ident,
    items: Vec<Item>,
}

impl Parse for ColocInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let block_name: syn::Ident = input.parse()?;
        let _: Token![!] = input.parse()?;

        let content;
        syn::bracketed!(content in input);

        let mut items = Vec::new();
        while !content.is_empty() {
            if content.peek(LitStr) {
                let lit: LitStr = content.parse()?;
                items.push(Item::Text(lit.value()));
            } else if content.peek(Token![_]) {
                let _: Token![_] = content.parse()?;
                items.push(Item::Placeholder);
            } else if content.peek(syn::Ident) {
                let name: syn::Ident = content.parse()?;
                let _: Token![!] = content.parse()?;

                let inner;
                if content.peek(syn::token::Bracket) {
                    syn::bracketed!(inner in content);
                } else if content.peek(syn::token::Paren) {
                    syn::parenthesized!(inner in content);
                } else {
                    syn::braced!(inner in content);
                }

                match name.to_string().as_str() {
                    "link" => {
                        let text: LitStr = inner.parse()?;
                        let _: Token![,] = inner.parse()?;
                        let href: LitStr = inner.parse()?;
                        if inner.peek(Token![,]) {
                            let _: Token![,] = inner.parse()?;
                        }
                        items.push(Item::Link {
                            text: text.value(),
                            href: href.value(),
                        });
                    }
                    other => {
                        return Err(syn::Error::new(
                            name.span(),
                            format!("unsupported macro in coloc!: {other}"),
                        ));
                    }
                }
            } else {
                return Err(content.error("expected string literal, macro call, or _"));
            }

            if content.peek(Token![,]) {
                let _: Token![,] = content.parse()?;
            }
        }

        Ok(ColocInput { block_name, items })
    }
}

#[derive(Clone)]
struct Slot {
    items: [Item; NUM_LANGS],
}

#[derive(Clone, Copy, PartialEq)]
enum Role {
    Predicate,
    Argument,
}

fn target_lang() -> usize {
    if cfg!(feature = "lang-ja") {
        JA
    } else if cfg!(feature = "lang-zh-hans") {
        ZH
    } else {
        EN
    }
}

const BASE_VERBS: &[&str] = &[
    "accept", "add", "allow", "apply", "are", "avoid",
    "become", "begin", "bring", "build",
    "call", "can", "change", "check", "choose", "click", "close",
    "come", "configure", "consider", "contain", "continue", "copy", "create",
    "define", "delete", "deploy", "describe", "design", "develop", "disable",
    "display", "do", "download",
    "edit", "enable", "ensure", "enter", "execute", "expect", "export", "extend",
    "feel", "find", "fix", "follow",
    "generate", "get", "give", "go",
    "handle", "has", "have", "help", "hold",
    "implement", "import", "improve", "include", "indicate", "initialize",
    "insert", "install", "integrate", "introduce", "invoke", "is",
    "keep", "know",
    "launch", "learn", "let", "like", "listen", "load", "look", "love",
    "maintain", "make", "manage", "mean", "modify", "move", "must",
    "need", "note",
    "offer", "open", "optimize", "output", "override",
    "parse", "pass", "perform", "place", "prefer", "prepare", "present",
    "prevent", "process", "produce", "provide", "publish", "put",
    "read", "receive", "recommend", "reduce", "register", "release",
    "remove", "render", "replace", "represent", "require", "resolve",
    "restart", "result", "return", "review", "run",
    "save", "search", "see", "select", "send", "serve", "set",
    "should", "show", "specify", "start", "stop", "store", "suggest",
    "support",
    "take", "tell", "test", "think", "try", "turn",
    "understand", "update", "upgrade", "upload", "use",
    "validate", "verify", "visit",
    "want", "work", "would", "write",
];

fn is_verb(word: &str) -> bool {
    let lower = word.to_lowercase();
    if BASE_VERBS.binary_search(&lower.as_str()).is_ok() {
        return true;
    }
    if let Some(stem) = lower.strip_suffix('s') {
        if BASE_VERBS.binary_search(&stem).is_ok() {
            return true;
        }
    }
    if let Some(stem) = lower.strip_suffix("es") {
        if BASE_VERBS.binary_search(&stem).is_ok() {
            return true;
        }
    }
    false
}

fn text_contains_verb(text: &str) -> bool {
    text.split_whitespace().any(|w| is_verb(w))
}

fn determine_role(slot: &Slot) -> Role {
    match &slot.items[EN] {
        Item::Text(t) if text_contains_verb(t) => Role::Predicate,
        _ => Role::Argument,
    }
}

fn reorder_slots(slots: Vec<Slot>, lang: usize) -> Vec<Slot> {
    if lang != JA {
        return slots;
    }

    let mut result = Vec::with_capacity(slots.len());
    let mut i = 0;
    while i < slots.len() {
        if i + 1 < slots.len()
            && determine_role(&slots[i]) == Role::Predicate
            && determine_role(&slots[i + 1]) == Role::Argument
        {
            result.push(slots[i + 1].clone());
            result.push(slots[i].clone());
            i += 2;
        } else {
            result.push(slots[i].clone());
            i += 1;
        }
    }
    result
}

fn item_to_tokens(item: &Item) -> TokenStream2 {
    match item {
        Item::Text(s) => {
            quote! { ::coloc::Inline::from(#s) }
        }
        Item::Link { text, href } => {
            quote! {
                ::coloc::Inline::Link {
                    href: #href.to_string(),
                    children: vec![::coloc::Inline::from(#text)],
                }
            }
        }
        Item::Placeholder => unreachable!("placeholders should be resolved before code generation"),
    }
}

#[proc_macro]
pub fn coloc(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ColocInput);
    let block_name = input.block_name.to_string();
    let items = input.items;

    if items.len() % NUM_LANGS != 0 {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "coloc! expects items in groups of {} (one per language), got {} items",
                NUM_LANGS,
                items.len()
            ),
        )
        .to_compile_error()
        .into();
    }

    let mut slots: Vec<Slot> = items
        .chunks(NUM_LANGS)
        .map(|chunk| Slot {
            items: [chunk[0].clone(), chunk[1].clone(), chunk[2].clone()],
        })
        .collect();

    for slot in &mut slots {
        let shared = slot
            .items
            .iter()
            .find(|i| !matches!(i, Item::Placeholder))
            .cloned();
        if let Some(shared) = shared {
            for item in &mut slot.items {
                if matches!(item, Item::Placeholder) {
                    *item = shared.clone();
                }
            }
        }
    }

    let lang = target_lang();
    let reordered = reorder_slots(slots, lang);

    let is_cjk = lang == JA || lang == ZH;
    let mut token_items: Vec<TokenStream2> = Vec::new();

    for (i, slot) in reordered.iter().enumerate() {
        if !is_cjk && i > 0 {
            token_items.push(quote! { ::coloc::Inline::from(" ") });
        }
        token_items.push(item_to_tokens(&slot.items[lang]));
    }

    let punct = if is_cjk { "。" } else { "." };
    token_items.push(quote! { ::coloc::Inline::from(#punct) });

    let output = match block_name.as_str() {
        "p" => quote! {
            ::coloc::Block::Paragraph(vec![#(#token_items),*])
        },
        other => {
            let msg = format!("unsupported block type in coloc!: {other}");
            quote! { compile_error!(#msg) }
        }
    };

    output.into()
}
