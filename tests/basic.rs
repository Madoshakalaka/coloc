use coloc::coloc;

#[test]
fn recommend_sentence() {
    let block = coloc!(p![
        "We recommend",
        "をお勧めします",
        "我们推荐",
        link!["Trunk", "https://trunkrs.dev/"],
        _,
        _,
    ]);

    #[cfg(not(any(feature = "lang-ja", feature = "lang-zh-hans")))]
    assert_eq!(
        block.to_markdown(),
        "We recommend [Trunk](https://trunkrs.dev/).\n\n"
    );

    #[cfg(feature = "lang-ja")]
    assert_eq!(
        block.to_markdown(),
        "[Trunk](https://trunkrs.dev/)をお勧めします。\n\n"
    );

    #[cfg(feature = "lang-zh-hans")]
    assert_eq!(
        block.to_markdown(),
        "我们推荐[Trunk](https://trunkrs.dev/)。\n\n"
    );
}
