// ## Map
// Role: msa/classic_continuity — 古典聖典連續性、雙語檔名、全體性與禁字之量測系統分析（MSA）零件。
// Inputs: 標準件 (Golden)、挑戰件 (Defects: 篇章缺失、末章截斷、漏頌、未閉合雙豎線、禁字違規、非雙語檔名)。
// Outputs: 參考量測結果、瑕疵件捕捉指標、鑑別力與重複性檢定。
// Invariants: 先驗量具再驗產品；鑑別力 >= 90.0%、重複性 = 1.00。

#![allow(dead_code)]

pub const RISK_TIER_STR: &str = "Strict";

pub struct MsaTestCase<'a> {
    pub name: &'a str,
    pub filename: &'a str,
    pub content: &'a str,
    pub expected_pass: bool,
    pub exp_chapters: Option<usize>,
    pub min_final_verse: Option<i32>,
    pub min_total_verses: Option<usize>,
}

pub const STANDARD_PARTS: &[MsaTestCase<'static>] = &[
    MsaTestCase {
        name: "Standard_1_Camatkara_Golden_Full",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 第一道月光開卷。\n\n## 第二道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 第二道月光開卷。\n\n## 第三道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 第三道月光開卷。\n\n## 第四道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 第四道月光開卷。\n\nText 2 ．第二頌\n॥ २ ॥\n譯文: 第二頌。\n\nText 3 ．第三頌\n॥ ३ ॥\n譯文: 第三頌圓滿終卷。\n",
        expected_pass: true,
        exp_chapters: Some(4),
        min_final_verse: Some(3),
        min_total_verses: Some(6),
    },
    MsaTestCase {
        name: "Standard_2_Simple_Continuous_3_Verses",
        filename: "Sample-Classic_示範聖典_繁體中文全譯本.md",
        content: "# Sample-Classic_示範聖典_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 一。\n\nText 2 ．第二頌\n॥ २ ॥\n譯文: 二。\n\nText 3 ．第三頌\n॥ ३ ॥\n譯文: 三。\n",
        expected_pass: true,
        exp_chapters: Some(1),
        min_final_verse: Some(3),
        min_total_verses: Some(3),
    },
];

pub const CHALLENGE_DEFECT_PARTS: &[MsaTestCase<'static>] = &[
    MsaTestCase {
        name: "Challenge_1_Incomplete_Starting_Verse",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 27 ．第二十七頌\n॥ २७ ॥\n譯文: 缺前導經文。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    MsaTestCase {
        name: "Challenge_2_Missing_Verse_Gap",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 一。\n\nText 5 ．第五頌\n॥ ५ ॥\n譯文: 缺 2..4。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    MsaTestCase {
        name: "Challenge_3_Unclosed_Danda",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\nयावत् किम् एतद् इति ता अहाहेति नोचुर्\n\n譯文: 缺少雙豎線閉合。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    MsaTestCase {
        name: "Challenge_4_Banned_Words_Guiyi_Pingtai",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 我們應當皈依至尊主，登上靈性平台，達到自我證悟境界。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    MsaTestCase {
        name: "Challenge_5_Non_Bilingual_Filename",
        filename: "奇妙月光_繁體中文全譯本.md",
        content: "# 奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 正常經文。\n",
        expected_pass: false,
        exp_chapters: Some(1),
        min_final_verse: Some(1),
        min_total_verses: Some(1),
    },
    MsaTestCase {
        name: "Challenge_6_Incomplete_Chapters",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 一。\n\n## 第二道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 二。\n",
        expected_pass: false,
        exp_chapters: Some(4),
        min_final_verse: Some(55),
        min_total_verses: Some(200),
    },
    MsaTestCase {
        name: "Challenge_7_Final_Chapter_Truncated",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 一。\n\n## 第二道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 二。\n\n## 第三道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 三。\n\n## 第四道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 四。\n\nText 30 ．第三十頌\n॥ ३० ॥\n譯文: 截斷於第 30 頌。\n",
        expected_pass: false,
        exp_chapters: Some(4),
        min_final_verse: Some(55),
        min_total_verses: Some(31),
    },
];
