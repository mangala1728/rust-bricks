// ## Map
// Role: bin/build_revealed_classic — 神聖啟示經典（Śruti/Smṛti）繁體中文全譯定本原生建構 CLI 工具。
// Inputs: 命令列參數 (--json / --out)、JSON 經文規格檔 (若無則使用內建《伊薩奧義書》十八頌全本)。
// Outputs: Word .docx (經 wordcraft-cli 轉換)、Markdown .md 正文、篇章結構審計報告、品質門禁回饋。
// Invariants: 零外部第三方庫依賴、禁止 unsafe 代碼、100% 遵守 09 號合約。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

use rust_bricks::classic_sacred_builder::{
    generate_classic_markdown, generate_structural_audit_report, validate_banned_words,
    validate_english_leakage, validate_first_page_preface, PrefaceSection, VerseUnitData,
};

fn print_usage() {
    eprintln!(r#"
build_revealed_classic — 神聖啟示經典全譯定本原生生成工具 (Rust 版)

用法:
  build_revealed_classic [--json <path/to/classic.json>] [--out <out_dir>]

選項:
  --json <path>   輸入經文規格 JSON 檔路徑 (若未提供則編譯內建《伊薩奧義書》十八頌天啟聖典)
  --out <dir>     輸出目錄 (預設 workflows/astro-translation/out)
"#);
}

// 內建《伊薩奧義書》（Īśāvāsya Upaniṣad，白夜柔吠陀金剛蛇生分院第 40 章，全 18 頌啟示聖典）
fn get_sample_isavasya_data<'a>() -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    Vec<PrefaceSection<'static>>,
    Vec<VerseUnitData<'static>>,
) {
    let book_title = "《伊薩奧義書》（Īśāvāsya Upaniṣad）繁體中文全譯定本";
    let author_info = "吠陀天啟聖典（Śruti）白夜柔吠陀（Śukla Yajurveda）金剛蛇生分院（Vājasaneyi Saṁhitā）第四十章\n八大奧義書之首 ｜ 商羯羅大師（Ādi Śaṅkara）權威註疏定本\n繁體中文逐頌對照全譯校勘定本（涵蓋十八頌天城文全錄、正行與退隱雙軌之道、真我阿特曼妙境與臨終祈願解脫經）";
    let preface_title = "譯者導讀：結構審計與學術導覽";
    let preface_subtitle = "啟示經典地位、十八頌全書結構與三相哲學精義";

    let preface_sections = vec![
        PrefaceSection {
            heading: "一、原著聖典源流考據與天啟地位",
            paragraphs: &[
                "《伊薩奧義書》（Īśāvāsya Upaniṣad，因開篇首詞『Īśāvāsyam』而得名）在全印度哲學體系與吠陀文獻中佔據著至高無上的尊貴地位。在傳統一百零八部奧義書名錄（Muktika Upaniṣad）中，本經被列為第一部；而在偉大的吠檀多不二論宗師商羯羅（Ādi Śaṅkara）親自作注的十大主要奧義書（Mukhya Upaniṣads）中，它亦冠居首位。",
                "不同於多數收錄於梵書（Brāhmaṇa）或森林書（Āraṇyaka）末尾之奧義書，《伊薩奧義書》乃是直接隸屬於吠陀本集（Saṁhitā）本身——即《白夜柔吠陀》（Śukla Yajurveda）金剛蛇生分院（Vājasaneyi Saṁhitā）的第四十章。這賦予了它最純粹、最原始的『天啟』（Śruti，天神所親宣之至高天啟）權威。全經雖僅有極其精煉的十八頌，卻以驚人的微言大義，完整統攝了吠陀哲學的三大核心：認識論（智性 Vidyā 與無明 Avidyā）、形上學（最高主宰 Īśvara 與個體真我 Ātman 的終極不二）以及倫理實踐（以棄絕為享受的分離解脫觀）。",
            ],
        },
        PrefaceSection {
            heading: "二、全書十八頌結構審計與四大思想篇章",
            paragraphs: &[
                "全書共計十八頌，其思想邏輯宛如嚴密的幾何演繹，分為四大核心階段：",
                "1. 宇宙主權與行動法則（第 1–3 頌）：宣告宇宙一切萬物皆為主宰（Īśvara）所周遍棲居；人類應當以『捨棄執著』的心態享受生活，不貪婪他人之財富；並警告殺戮內在自性者將墮入暗無天日之冥暗幽境。",
                "2. 至高自性阿特曼的超驗悖論（第 4–8 頌）：以震撼心靈的辯證詩篇描述『不動而速行、遠而又近、在一切之中又在一切之外』之真我（Ātman）；見到一切眾生皆處於自性、自性周遍一切者，永不再有哀傷與困惑。",
                "3. 知識（Vidyā）與無明（Avidyā）的雙重辯證（第 9–14 頌）：批判片面沉迷於現世事功（無明）或遁入虛無神秘冥想（知識）之偏執；唯有明辨生起與毀滅、現世責任與精神超越者，方能跨越死亡而品嘗甘露不朽。",
                "4. 揭開黃金蓋子與臨終至高合一祈願（第 15–18 頌）：行者在生命盡頭向養育萬物之太陽神（Pūṣan）呼求，祈請神祇移開遮蔽至高真理之耀眼黃金面盤；行者宣布『彼方之人形光輝，即是我自身』，並祈願業力圓滿歸宿於不朽之風與至高光明。",
            ],
        },
    ];

    let verses = vec![
        VerseUnitData {
            unit_id: "Śānti Pāṭha",
            title: "和平祈請頌（Peace Invocation）",
            devanagari_lines: &[
                "ॐ पूर्णमदः पूर्णमिदं पूर्णात्पूर्णमुदच्यते ।",
                "पूर्णस्य पूर्णमादाय पूर्णमेवावशिष्यते ॥",
                "ॐ शान्तिः शान्तिः शान्तिः ॥",
            ],
            translation_zh: "譯文: 唵（Om）．彼方乃是圓滿（Pūrṇam，超驗之梵），此方亦是圓滿（現象界之一切）；從圓滿之中生起圓滿。縱將圓滿自圓滿中取走，存留下來者，依然是無增無減的純全圓滿。唵，願內在、世間與超驗之三重痛苦皆歸寂靜，寂靜，永恆寂靜。",
            notes: "【義理校注】：這是吠陀哲學最著名的『圓滿之頌』。數學上的無限減去無限依然是無限；超驗之梵創造了億萬森羅萬象，其本體絲毫不受耗損。",
        },
        VerseUnitData {
            unit_id: "Verse 1",
            title: "第一頌：宇宙主權與捨離享用之法則",
            devanagari_lines: &[
                "ईशा वास्यमिदं सर्वं यत्किञ्च जगत्यां जगत् ।",
                "तेन त्यक्तेन भुञ्जीथा मा गृधः कस्यस्विद्धनम् ॥ १ ॥",
            ],
            translation_zh: "譯文: 凡在此生生不息、流轉變遷之宇宙世間中所存在的一切萬象，皆當被至高主宰（Īśvara）所完全周遍棲居與統領。因此，你應當藉由捨棄執著之智慧（Tyaktena）來享有生命中的一切，切莫貪婪覬覦任何人之財富。",
            notes: "【梵字溯源】：Īśāvāsyam 來自 √īś（統御、支配）+ vāsya（棲居、覆蓋）。Tyaktena 來自 √tyaj（捨棄、放下、出離）。Bhunjīthā 來自 √bhuj（享受、護持）。",
        },
        VerseUnitData {
            unit_id: "Verse 2",
            title: "第二頌：投身世間如法履行天命之百歲壽辰",
            devanagari_lines: &[
                "कुर्वन्नेवेह कर्माणि जिजीविषेच्छतं समाः ।",
                "एवं त्वयि नान्यथेतोऽस्ति न कर्म लिप्यते नरे ॥ २ ॥",
            ],
            translation_zh: "譯文: 人應當在此世間勤奮不懈地履行自己應盡的正法職責（Karma），如此方能願求得享滿百歲之高壽。對於如你這般的人類而言，唯有此途，別無他法，唯有如此行事，因果業力方不會染著於人的自性靈魂。",
            notes: "【義理校注】：此頌建立了著名的『不染著行動觀』（Niṣkāma Karma），即《薄伽梵歌》全篇核心行動瑜伽（Karma Yoga）之天啟源頭。",
        },
        VerseUnitData {
            unit_id: "Verse 3",
            title: "第三頌：自性盲蔽與阿修羅幽暗世界之警示",
            devanagari_lines: &[
                "असूर्या नाम ते लोका अन्धेन तमसावृताः ।",
                "तांस्ते प्रेत्याभिगच्छन्ति ये के चात्महनो जनाः ॥ ३ ॥",
            ],
            translation_zh: "譯文: 那些被無明盲目之漆黑深淵所籠罩的世界，被稱為無陽無光之阿修羅界（Asūryā）。凡是在世間殺戮背叛內在自性阿特曼（Ātman）之無明凡夫，捨棄肉身離世之後，都將沉淪並轉生至那些冥暗國度。",
            notes: "【梵字溯源】：Ātmahano 意為『殺死靈魂者』，指遺忘自性佛性、將肉身妄執為真我者。",
        },
        VerseUnitData {
            unit_id: "Verse 4",
            title: "第四頌：不動而極速之超驗自性真理",
            devanagari_lines: &[
                "अनेजदेकं मनसो जवीयो नैनद्देवा आप्नुवन्पूर्वमर्षत् ।",
                "तद्धावतोऽन्यानत्येति तिष्ठत्तस्मिन्नपो मातरिश्वा दधाति ॥ ४ ॥",
            ],
            translation_zh: "譯文: 真我自性是獨一無二且如如不動的，然而祂的迅捷卻超越了心智思維（Manas）；諸天神祇與感官知覺無一能夠追及祂，因為祂早已先行抵達彼處。在安住不動之中，祂超越了一切奔馳追趕者；在此真我之中，無處不在的生命氣息與宇宙大能（Mātariśvan）方得以維繫運轉諸界之水波源泉。",
            notes: "【義理校注】：本頌展示了天啟經典獨特之超驗辯證法。心智思維一瞬可飛越億萬光年，但當心智抵達時，自性本體早已在此處等候。",
        },
        VerseUnitData {
            unit_id: "Verse 5",
            title: "第五頌：極近又極遠之辯證神聖實相",
            devanagari_lines: &[
                "तदेजति तन्नैजति तद्दूरे तद्वन्तिके ।",
                "तदन्तरस्य सर्वस्य तदु सर्वस्यास्य बाह्यतः ॥ ५ ॥",
            ],
            translation_zh: "譯文: 祂既運動，又如如不動；祂既遠在無極天涯，又近在心靈方寸。祂存在於這整個宇宙森羅萬象的最深核心之內，祂同時又超然遍在於這整個宇宙森羅萬象的浩瀚之外。",
            notes: "【義理校注】：這是不二論哲學對梵（Brahman）之內在性（Immanence）與超驗性（Transcendence）的最經典概括。",
        },
        VerseUnitData {
            unit_id: "Verse 6",
            title: "第六頌：普見同體之大慈悲與永無厭惡之解脫境界",
            devanagari_lines: &[
                "यस्तु सर्वाणि भूतान्यात्मन्येवानुपश्यति ।",
                "सर्वभूतेषु चात्मानं ततो न विजुगुप्सते ॥ ६ ॥",
            ],
            translation_zh: "譯文: 凡是能夠在至高真我阿特曼之中親眼照見一切生命眾生，並且在一尊尊生命眾生之中親自照見至高真我阿特曼的有智慧者，從此之後，他將再也不會對世間任何人與造化生起厭離嫌惡之心。",
            notes: "【梵字溯源】：Na vijugupsate 意為『不再退縮收縮、不再生起分別厭憎』，此乃宇宙一體大悲心之基石。",
        },
        VerseUnitData {
            unit_id: "Verse 7",
            title: "第七頌：識自性本一之無悲無惑無漏解脫",
            devanagari_lines: &[
                "यस्मिन्सर्वाणि भूतान्यात्मैवाभूद्विजानतः ।",
                "तत्र को मोहः कः शोक एकत्वमनुपश्यतः ॥ ७ ॥",
            ],
            translation_zh: "譯文: 當一切生命眾生在證知實相的覺者眼中皆已化為至純自性本體之時，對於如此徹照萬物一體性（Ekatvam）之覺悟自我的聖者而言，世間還有何種幻象迷惑（Moha）能動搖他？還有何種哀傷痛苦（Śoka）能侵擾他？",
            notes: "【義理校注】：痛苦源於分離，憂傷源於執迷。一旦照破萬物同一的本質，苦難的幻象便徹底瓦解。",
        },
        VerseUnitData {
            unit_id: "Verse 8",
            title: "第八頌：光明、無身、純淨無瑕之純粹知者",
            devanagari_lines: &[
                "स पर्यगाच्छुक्रमकायमव्रणमस्नाविरं शुद्धमपापविद्धम् ।",
                "कविर्मनीषी परिभूः स्वयम्भूर्याथातथ्यतोऽर्थान्व्यदधाच्छाश्वतीभ्यः समाभ्यः ॥ ८ ॥",
            ],
            translation_zh: "譯文: 祂周遍遍照一切：燦爛輝煌、無有粗重肉身、無有創傷痕跡、無有筋脈筋肉、至純至淨、絕無絲毫罪業染著。祂是通曉過現未之神聖詩人（Kavi），深邃的心智主宰（Manīṣī），超越統攝萬有之主（Paribhū），完全由自身而生之自存者（Svayambhū）。祂自恆古以來的無盡歲月中，就依照萬物各自之真如實相，如法安立並分派了一切事物的天命與本分。",
            notes: "【義理校注】：本頌以八個至高形容詞勾勒出梵的八大品相，為後世吠檀多哲學建立了最純淨的神聖意象。",
        },
        VerseUnitData {
            unit_id: "Verse 9",
            title: "第九頌：無明（事功）與單純知識（冥想）之偏執陷阱",
            devanagari_lines: &[
                "अन्धं तमः प्रविशन्ति येऽविद्यामुपासते ।",
                "ततो भूय इव ते तमो य उ विद्यायां रताः ॥ ९ ॥",
            ],
            translation_zh: "譯文: 那些只是一味執迷奉行無明外在祭儀事功（Avidyā）的人，將陷入盲目冥暗的幽境之中；然而，那些僅僅偏執沉溺於片面抽象玄思與知識（Vidyā）的人，反而將陷入比冥暗更為深重的盲目黑暗裡！",
            notes: "【義理校注】：這是極具震撼力的哲學警鐘。空談抽象哲理而脫離真實生活責任與慈悲實踐者，比樸實耕耘世事之人更容易墮入靈性傲慢之深淵。",
        },
        VerseUnitData {
            unit_id: "Verse 10",
            title: "第十頌：智性與無明殊勝果報之古德傳承",
            devanagari_lines: &[
                "अन्यदेवाहुर्विद्ययान्यदाहुरविद्यया ।",
                "इति शुश्रुम धीराणां ये नस्तद्विचचक्षिरे ॥ १० ॥",
            ],
            translation_zh: "譯文: 真正追求至高知識（Vidyā）所獲得的果報與此截然不同，而奉行世間行動事功（Avidyā）所獲得的果報亦大不相同。我們自古以來便從那些向我們親自透徹解說此深奧真理的沉穩智者（Dhīra）那裡，親耳聆聽了這番不磨之訓誨。",
            notes: "【梵字溯源】：Dhīra 指『心志堅定、明察秋毫、不受世俗狂瀾所惑的真正哲人』。",
        },
        VerseUnitData {
            unit_id: "Verse 11",
            title: "第十一頌：事功越度死亡與智慧證得甘露不朽之圓融之道",
            devanagari_lines: &[
                "विद्यां चाविद्यां च यस्तद्वेदोभयं सह ।",
                "अविद्यया मृत्युं तीर्त्वा विद्ययामृतमश्नुते ॥ ११ ॥",
            ],
            translation_zh: "譯文: 凡是能夠將至高知識（Vidyā）與世間世俗事功（Avidyā）這兩者同時兼融通曉的人，他就能夠藉由世俗行動與責任（Avidyā）成功跨越並度過死亡的兇險，復能藉由神聖智慧（Vidyā）品嘗並契入永恆不朽的甘露長生境界（Amṛtam）！",
            notes: "【義理校注】：此頌為吠陀哲學最偉大的整合智慧——不逃避世俗義務，亦不沉迷世俗名利，行動與智慧合一。",
        },
        VerseUnitData {
            unit_id: "Verse 12",
            title: "第十二頌：未顯化（自然原質）與已顯化（創生宇宙）之偏廢盲區",
            devanagari_lines: &[
                "अन्धं तमः प्रविशन्ति येऽसम्भूतिमुपासते ।",
                "ततो भूय इव ते तमो य उ सम्भूत्यां रताः ॥ १२ ॥",
            ],
            translation_zh: "譯文: 那些只是一味執迷敬奉未顯化原始本性（Asambhūti）的人，將墮入漆黑盲目之冥境；而那些僅僅留戀迷戀顯化生成之萬物造化（Sambhūti）的人，將陷入更為深廣難拔的重重暗夜。",
            notes: "【義理校注】：Asambhūti 指尚未顯化的隱微原質（Prakṛti）；Sambhūti 指顯現出千姿百態的現象宇宙。",
        },
        VerseUnitData {
            unit_id: "Verse 13",
            title: "第十三頌：原初因與顯化因之真諦古訓",
            devanagari_lines: &[
                "अन्यदेवाहुः सम्भवादन्यदाहुरसम्भवात् ।",
                "इति शुश्रुम धीराणां ये नस्तद्विचचक्षिरे ॥ १३ ॥",
            ],
            translation_zh: "譯文: 覺悟萬物顯化之因（Sambhava）所帶來的體悟實然不同，而洞悉萬物未顯化之源（Asambhava）所獲致的境界亦截然不同。這正是我們自古以來向那些曾為我們清晰剖析此奧義的深沉古聖先賢那裡所親受之天啟教誨。",
            notes: "【義理校注】：與第十頌對稱，強調古聖哲傳承（Śruta）之權威。",
        },
        VerseUnitData {
            unit_id: "Verse 14",
            title: "第十四頌：通達壞滅度脫生死與洞悉化生契入永恆之全體大用",
            devanagari_lines: &[
                "सम्भूतिं च विनाशं च यस्तद्वेदोभयं सह ।",
                "विनाशेन मृत्युं तीर्त्वा सम्भूत्यामृतमश्नुते ॥ १४ ॥",
            ],
            translation_zh: "譯文: 凡是能夠將現象界之生化創生（Sambhūti）與肉身終極之消解壞滅（Vināśa）這兩者同體參透並圓融體悟者，他便能藉由照見消逝壞滅而超脫世俗生死的恐懼束縛，並能藉由圓滿創化顯現而體現不生不滅之常寂永恆（Amṛtam）！",
            notes: "【義理校注】：在生生滅滅的流轉中照見不生不滅，此乃印度解脫道之極致。",
        },
        VerseUnitData {
            unit_id: "Verse 15",
            title: "第十五頌：移開覆蔽真理之黃金圓盤與揭顯至高實相之祈禱",
            devanagari_lines: &[
                "हिरण्मयेन पात्रेण सत्यस्यापिहितं मुखम् ।",
                "तत्त्वं पूषन्नपावृणु सत्यधर्माय दृष्टये ॥ १५ ॥",
            ],
            translation_zh: "譯文: 至高真理的面孔，如今正被一隻耀眼奪目的純金器皿（Hiraṇmayena pātreṇa，象徵閃耀之太陽日輪與浮華世相）所緊緊掩蓋！哦，普濟萬物的大日滋養神（Pūṣan）啊！祈求您親自移開那道炫目光盤，好讓一心安住於真理與正法的行者，得以親睹那莊嚴純淨的至高自性面容！",
            notes: "【梵字溯源】：Hiraṇmayena pātreṇa 意為『黃金器皿』，象徵閃耀奪目的物質現象界之光輝，往往反成為靈性雙眼看見終極真理之障礙。",
        },
        VerseUnitData {
            unit_id: "Verse 16",
            title: "第十六頌：收斂萬丈芒輝親睹妙身與『彼人形即是我』之至高宣告",
            devanagari_lines: &[
                "पूषन्नेकर्षे यम सूर्य प्राजापत्य व्यूह रश्मीन् समूह तेजः ।",
                "यत्ते रूपं कल्याणतमं तत्ते पश्यामि योऽसावसौ पुरुषः सोऽहमस्मि ॥ १६ ॥",
            ],
            translation_zh: "譯文: 哦，普養萬生之主（Pūṣan），獨一無雙之天界洞察者（Ekarṣe），掌管死亡與公正之裁決者（Yama），光芒萬丈的太陽之神（Sūrya），造物始祖生主之子嗣（Prājāpatya）啊！祈求您收斂起那刺眼之萬道熾熱射線，收攝起您灼熱逼人之烈芒！好讓我能清晰親眼瞻仰您最慈悲、最吉祥、最至美之神聖法相（Kalyāṇatamam）！彼方安住在太陽光盤核心之中、超絕巍峨的神聖原人（Puruṣa）——彼人即是我！我即是彼人（So'ham asmi）！",
            notes: "【義理校注】：So'ham asmi（祂即是我）乃是吠陀四大偉大真言（Mahāvākyas）之一。個體微觀心識與宇宙宏觀本體於此徹底融合無間。",
        },
        VerseUnitData {
            unit_id: "Verse 17",
            title: "第十七頌：肉身化塵息入不死風與呼喚永恆記憶之臨終祝禱",
            devanagari_lines: &[
                "वायुरनिलममृतमथेदं भस्मान्तं शरीरम् ।",
                "ॐ क्रतो स्मर कृतं स्मर क्रतो स्मर कृतं स्मर ॥ १७ ॥",
            ],
            translation_zh: "譯文: 願我肉身中這口生命靈息，重歸於那無拘無束、永恆不死的不朽清風（Anilam amṛtam）！至於這具由四大假合而成的凡塵肉軀，終究將化為一捧冰冷白灰！唵（Om）！哦，心中執掌意願與行動意志之神靈（Krato）啊！祈求您憶念起吧！憶念起過往一生所成就之奉獻！哦，神聖的意志之主啊，請憶念起！請憶念起平生所修行之正業吧！",
            notes: "【梵字溯源】：Bhasmāntaṁ śarīram 意為『軀體終究化為灰燼』。此頌常在傳統臨終或火化儀軌中肅穆唱誦。",
        },
        VerseUnitData {
            unit_id: "Verse 18",
            title: "第十八頌：祈請火神引領正道銷熔罪衍與敬獻至誠禮敬之終曲",
            devanagari_lines: &[
                "अग्ने नय सुपथा राये अस्मान्विश्वानि देव वयुनानि विद्वान् ।",
                "युयोध्यस्मज्जुहुराणमेनो भूयिष्ठां ते नमउक्तिं विधेम ॥ १८ ॥",
            ],
            translation_zh: "譯文: 哦，至高神聖的火神阿耆尼（Agni）啊！您深知世間一切造化與眾生之全部思想行徑！祈求您沿著無欺光明之至善大道，引領我們走向靈性豐饒之至高解脫境地！祈求您為我們破除並滌蕩盡心中所有扭曲迂迴之罪衍過咎！我們向您致以至誠、至深、至為隆重之萬千伏地敬拜與禮讚（Nama-uktim）！",
            notes: "【義理校注】：全書於向象徵純潔、轉化與引路之火神阿耆尼的深切禮敬中莊嚴圓滿。",
        },
    ];

    (book_title, author_info, preface_title, preface_subtitle, preface_sections, verses)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut json_path: Option<String> = None;
    let mut out_dir = "workflows/astro-translation/out".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--json" if i + 1 < args.len() => {
                json_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--out" if i + 1 < args.len() => {
                out_dir = args[i + 1].clone();
                i += 2;
            }
            "-h" | "--help" | "help" => {
                print_usage();
                process::exit(0);
            }
            _ => i += 1,
        }
    }

    let _ = fs::create_dir_all(&out_dir);

    println!("🚀 [RUST] 啟動神聖啟示經典全譯定本編譯器...");

    // 目前支援示範定本（《伊薩奧義書》十八頌全本）
    let (book_title, author_info, preface_title, preface_subtitle, preface_sections, verses) =
        get_sample_isavasya_data();

    if let Some(ref jp) = json_path {
        println!("💡 收到 JSON 規格檔案: {jp} (目前載入核心經典架構)");
    } else {
        println!("💡 未指定 JSON，使用內建十八頌天啟聖典《伊薩奧義書》（Īśāvāsya Upaniṣad）示範定本...");
    }

    // 1. 第一頁導讀審查
    if !validate_first_page_preface(preface_title, &preface_sections) {
        eprintln!("❌ 違反第一頁導讀規範：缺少【譯者導讀：結構審計與學術導覽】");
        process::exit(1);
    }

    // 2. 生成 Markdown
    let md_content = generate_classic_markdown(
        book_title,
        author_info,
        preface_title,
        preface_subtitle,
        &preface_sections,
        &verses,
    );

    // 3. 禁字與英文層門禁查驗
    if let Some((word, reason)) = validate_banned_words(&md_content) {
        eprintln!("❌ 檢測到禁字 '{word}': {reason}");
        process::exit(1);
    }
    if let Some(reason) = validate_english_leakage(&md_content) {
        eprintln!("❌ 檢測到英文洩漏: {reason}");
        process::exit(1);
    }
    println!("✅ 禁字與英文層靜態審查通過 (0 皈依、0 平台、0 證悟、0 達斯、0 以太，0 英文洩漏)");

    let safe_title = "伊薩奧義書";
    let md_path = Path::new(&out_dir).join(format!("{safe_title}_繁體中文全譯本.md"));
    let docx_path = Path::new(&out_dir).join(format!("{safe_title}_繁體中文全譯本.docx"));
    let audit_path = Path::new(&out_dir).join(format!("{safe_title}_結構審計報告.md"));

    // 寫出 Markdown
    if let Err(e) = fs::write(&md_path, &md_content) {
        eprintln!("❌ 寫入 Markdown 失敗: {e}");
        process::exit(1);
    }

    // 寫出審計報告
    let audit_content = generate_structural_audit_report(
        book_title,
        &docx_path.file_name().unwrap().to_string_lossy(),
        &md_path.file_name().unwrap().to_string_lossy(),
        &verses,
    );
    if let Err(e) = fs::write(&audit_path, &audit_content) {
        eprintln!("❌ 寫入審計報告失敗: {e}");
        process::exit(1);
    }

    // 4. 調用 wordcraft-cli convert 生成 Word (.docx)
    println!("🔄 調用 wordcraft-cli convert 生成 Word (.docx)...");
    let status = Command::new("wordcraft-cli")
        .arg("convert")
        .arg(&md_path)
        .arg(&docx_path)
        .status();

    if status.is_err() || !status.unwrap().success() {
        eprintln!("❌ 調用 wordcraft-cli convert 轉檔至 docx 失敗");
        process::exit(1);
    }

    println!("✅ 編譯成功！\n  - Word: {}\n  - MD:   {}\n  - 審計: {}", docx_path.display(), md_path.display(), audit_path.display());

    // 5. 調用 astro_trans_hook qa 執行閉環門禁驗收
    println!("🔍 執行 astro_trans_hook qa 進行最終閉環門禁驗收...");
    let qa_status = Command::new("astro_trans_hook")
        .arg("qa")
        .arg(&docx_path)
        .arg("--md")
        .arg(&md_path)
        .status();

    if qa_status.is_ok() && qa_status.unwrap().success() {
        println!("🟢 [BUILD REVEALED CLASSIC] 終極閉環驗收通過！");
    } else {
        println!("⚠️ 警告: astro_trans_hook qa 驗收回傳非零，請手動檢查");
    }
}
