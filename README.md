# coloc

Co-located i18n for styled strings in Rust.

**Status: Proof of Concept**

## Motivation

Traditional i18n solutions require you to repeat the structural markup of every styled string in every language file. Consider a paragraph with a link:

```md
We recommend [Trunk](https://trunkrs.dev/).
```

In a typical i18n setup, the Japanese and Chinese translation files each duplicate the full structure:

```md
<!-- ja.md -->
[Trunk](https://trunkrs.dev/)をお勧めします。

<!-- zh-Hans.md -->
我们推荐[Trunk](https://trunkrs.dev/)。
```

The link markup `[Trunk](https://trunkrs.dev/)` is copied verbatim into every file. For pages with dozens of links, code blocks, bold text, and nested formatting, this structural repetition becomes a maintenance burden. Changing a URL or renaming a project means updating every language file. Translators must carefully preserve markup they don't understand, and reviewers must diff structural noise to find actual translation changes.

## How coloc works

coloc eliminates structural repetition by co-locating all translations in a single DSL expression. Shared elements like links are written once, and the `coloc!` macro handles word order and formatting per language:

```rust
coloc!(p![
    "We recommend",                          // en
    "をお勧めします",                          // ja
    "我们推荐",                               // zh-Hans
    link!["Trunk", "https://trunkrs.dev/"],  // shared across all languages
    _,                                       // ja: reuse link
    _,                                       // zh-Hans: reuse link
])
```

The macro analyzes the English text to detect its grammatical role (subject+verb, object, etc.) and applies language-appropriate word order:

- **English** (SVO): `We recommend [Trunk](https://trunkrs.dev/).`
- **Japanese** (SOV): `[Trunk](https://trunkrs.dev/)をお勧めします。`
- **Chinese** (SVO): `我们推荐[Trunk](https://trunkrs.dev/)。`

Spacing and punctuation are inserted automatically: English gets spaces between elements and a period; CJK languages get no spaces and an ideographic full stop.

## Compile-time language selection

Only one language is packed into the binary. Select via Cargo feature flags:

```sh
cargo build --features lang-en
cargo build --features lang-ja
cargo build --features lang-zh-hans
```

No feature flag defaults to English. Enabling multiple non-English features is a compile error.

## Supported languages

- English (`lang-en`)
- Japanese (`lang-ja`)
- Simplified Chinese (`lang-zh-hans`)

## Roadmap

- **LLM-based grammatical detection**: Replace the built-in verb list with LLM analysis to determine what word order transformation to apply. Results would be cached on the filesystem so the LLM is only consulted once per unique English string.
- **More block/inline types**: Headings, bold, italic, code blocks, lists.
- **HTML output**: Dual rendering to both Markdown and HTML, following the pattern in [yew-rs](https://github.com/yewstack/yew/pull/4069).
