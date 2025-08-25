
export type HTMLText = string;
export type RefIdStr = string;
export type DateStr = string;

type AssetData = {
    path?: string;
    aliases: Map<string, string>;
};

export type ConfigData = {
    name: string,
    description?: HTMLText,
    authors?: string[],
    license?: string,
    assets?: AssetData,
    source?: string,
    language?: LanguageCode,
    publication_year?: DateStr,
};

// Bible Module
export type BibleConfig = ConfigData;
export type VerseEntry = {
    verse_id: VerseIdStr,
    words: Word[],
};

export type Word = {
    word: string,
    begin_punc?: string,
    end_punc?: string,
    italics?: boolean,
    red?: boolean,
};

// Strong's Definition Module
export type StrongsDefinitionsConfig = ConfigData;
export type StrongsEntry = {
    strongs: StrongsNumber,
    definition: HTMLText,
}

// Strongs Linker Module
export type StrongsLinkerConfig = ConfigData & {
    bible: string | null,
};

export type WordRange = `${number}-${number}` | `${number}`;
export type StrongsWord = {
    strongs: StrongsNumber,
    range: WordRange,
}

export type StrongsLinkerEntry = {
    verse_id: VerseIdStr,
    words: StrongsWord[],
}

// Dictionary Module
export type DictionaryConfig = ConfigData;
export type DictionaryEntry = {
    term: string,
    definition: HTMLText,
    id: number,
}

// Cross References
export type CrossReferenceConfig = ConfigData & {
    bible: string | null,
}

export type CrossReferenceEntry = |{
    type: 'directed',
    source: RefIdStr,
    targets: RefIdStr[],
    id: number,
    note?: HTMLText,
}| {
    type: 'mutual',
    references: RefIdStr[],
    id: number,
    note?: HTMLText,
}

// Commentary
export type CommentaryConfig = ConfigData & {
    bible: string | null,
}

export type CommentaryEntry = {
    references: RefIdStr[],
    id: number,
    note: HTMLText,
}

// Misc
export type VerseIdStr = `${OsisBook}.${number}.${number}`;
export type StrongsLanguage = 'H' | 'G';
export type StrongsNumber = `${StrongsLanguage}${number}`;


export type OsisBook =
  | "Gen"
  | "Exod"
  | "Lev"
  | "Num"
  | "Deut"
  | "Josh"
  | "Judg"
  | "Ruth"
  | "1Sam"
  | "2Sam"
  | "1Kgs"
  | "2Kgs"
  | "1Chr"
  | "2Chr"
  | "Ezra"
  | "Neh"
  | "Esth"
  | "Job"
  | "Ps"
  | "Prov"
  | "Eccl"
  | "Song"
  | "Isa"
  | "Jer"
  | "Lam"
  | "Ezek"
  | "Dan"
  | "Hos"
  | "Joel"
  | "Amos"
  | "Obad"
  | "Jonah"
  | "Mic"
  | "Nah"
  | "Hab"
  | "Zeph"
  | "Hag"
  | "Zech"
  | "Mal"
  | "Matt"
  | "Mark"
  | "Luke"
  | "John"
  | "Acts"
  | "Rom"
  | "1Cor"
  | "2Cor"
  | "Gal"
  | "Eph"
  | "Phil"
  | "Col"
  | "1Thess"
  | "2Thess"
  | "1Tim"
  | "2Tim"
  | "Titus"
  | "Phlm"
  | "Heb"
  | "Jas"
  | "1Pet"
  | "2Pet"
  | "1John"
  | "2John"
  | "3John"
  | "Jude"
  | "Rev"

export type LanguageCode =
  | "aa" // Afar
  | "ab" // Abkhazian
  | "ae" // Avestan
  | "af" // Afrikaans
  | "ak" // Akan
  | "am" // Amharic
  | "an" // Aragonese
  | "ar" // Arabic
  | "as" // Assamese
  | "av" // Avaric
  | "ay" // Aymara
  | "az" // Azerbaijani
  | "ba" // Bashkir
  | "be" // Belarusian
  | "bg" // Bulgarian
  | "bh" // Bihari
  | "bi" // Bislama
  | "bm" // Bambara
  | "bn" // Bengali
  | "bo" // Tibetan
  | "br" // Breton
  | "bs" // Bosnian
  | "ca" // Catalan
  | "ce" // Chechen
  | "ch" // Chamorro
  | "co" // Corsican
  | "cr" // Cree
  | "cs" // Czech
  | "cu" // Church Slavic
  | "cv" // Chuvash
  | "cy" // Welsh
  | "da" // Danish
  | "de" // German
  | "dv" // Divehi
  | "dz" // Dzongkha
  | "ee" // Ewe
  | "el" // Greek
  | "en" // English
  | "eo" // Esperanto
  | "es" // Spanish
  | "et" // Estonian
  | "eu" // Basque
  | "fa" // Persian
  | "ff" // Fulah
  | "fi" // Finnish
  | "fj" // Fijian
  | "fo" // Faroese
  | "fr" // French
  | "fy" // Western Frisian
  | "ga" // Irish
  | "gd" // Scottish Gaelic
  | "gl" // Galician
  | "gn" // Guarani
  | "gu" // Gujarati
  | "gv" // Manx
  | "ha" // Hausa
  | "he" // Hebrew
  | "hi" // Hindi
  | "ho" // Hiri Motu
  | "hr" // Croatian
  | "ht" // Haitian
  | "hu" // Hungarian
  | "hy" // Armenian
  | "hz" // Herero
  | "ia" // Interlingua
  | "id" // Indonesian
  | "ie" // Interlingue
  | "ig" // Igbo
  | "ii" // Sichuan Yi
  | "ik" // Inupiaq
  | "io" // Ido
  | "is" // Icelandic
  | "it" // Italian
  | "iu" // Inuktitut
  | "ja" // Japanese
  | "jv" // Javanese
  | "ka" // Georgian
  | "kg" // Kongo
  | "ki" // Kikuyu
  | "kj" // Kuanyama
  | "kk" // Kazakh
  | "kl" // Kalaallisut
  | "km" // Khmer
  | "kn" // Kannada
  | "ko" // Korean
  | "kr" // Kanuri
  | "ks" // Kashmiri
  | "ku" // Kurdish
  | "kv" // Komi
  | "kw" // Cornish
  | "ky" // Kyrgyz
  | "la" // Latin
  | "lb" // Luxembourgish
  | "lg" // Ganda
  | "li" // Limburgan
  | "ln" // Lingala
  | "lo" // Lao
  | "lt" // Lithuanian
  | "lu" // Luba-Katanga
  | "lv" // Latvian
  | "mg" // Malagasy
  | "mh" // Marshallese
  | "mi" // Māori
  | "mk" // Macedonian
  | "ml" // Malayalam
  | "mn" // Mongolian
  | "mr" // Marathi
  | "ms" // Malay
  | "mt" // Maltese
  | "my" // Burmese
  | "na" // Nauru
  | "nb" // Norwegian Bokmål
  | "nd" // North Ndebele
  | "ne" // Nepali
  | "ng" // Ndonga
  | "nl" // Dutch
  | "nn" // Norwegian Nynorsk
  | "no" // Norwegian
  | "nr" // South Ndebele
  | "nv" // Navajo
  | "ny" // Chichewa
  | "oc" // Occitan
  | "oj" // Ojibwa
  | "om" // Oromo
  | "or" // Oriya
  | "os" // Ossetian
  | "pa" // Punjabi
  | "pi" // Pali
  | "pl" // Polish
  | "ps" // Pashto
  | "pt" // Portuguese
  | "qu" // Quechua
  | "rm" // Romansh
  | "rn" // Rundi
  | "ro" // Romanian
  | "ru" // Russian
  | "rw" // Kinyarwanda
  | "sa" // Sanskrit
  | "sc" // Sardinian
  | "sd" // Sindhi
  | "se" // Northern Sami
  | "sg" // Sango
  | "si" // Sinhala
  | "sk" // Slovak
  | "sl" // Slovenian
  | "sm" // Samoan
  | "sn" // Shona
  | "so" // Somali
  | "sq" // Albanian
  | "sr" // Serbian
  | "ss" // Swati
  | "st" // Southern Sotho
  | "su" // Sundanese
  | "sv" // Swedish
  | "sw" // Swahili
  | "ta" // Tamil
  | "te" // Telugu
  | "tg" // Tajik
  | "th" // Thai
  | "ti" // Tigrinya
  | "tk" // Turkmen
  | "tl" // Tagalog
  | "tn" // Tswana
  | "to" // Tongan
  | "tr" // Turkish
  | "ts" // Tsonga
  | "tt" // Tatarcd
  | "tw" // Twi
  | "ty" // Tahitian
  | "ug" // Uighur
  | "uk" // Ukrainian
  | "ur" // Urdu
  | "uz" // Uzbek
  | "ve" // Venda
  | "vi" // Vietnamese
  | "vo" // Volapük
  | "wa" // Walloon
  | "wo" // Wolof
  | "xh" // Xhosa
  | "yi" // Yiddish
  | "yo" // Yoruba
  | "za" // Zhuang
  | "zh" // Chinese
  | "zu"; // Zulu
