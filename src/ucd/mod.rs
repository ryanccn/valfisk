// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

mod data;
pub use data::*;

// ── Notation ──────────────────────────────────────────────────────────────────

/// Format a codepoint as `U+XXXX` (minimum 4 hex digits).
pub fn unicode_notation(c: char) -> String {
    format!("U+{:04X}", c as u32)
}

// ── Character name ────────────────────────────────────────────────────────────

/// Return the Unicode character name, or `None` for surrogates / private-use /
/// unassigned code points without formal names.
pub fn name_of(c: char) -> Option<String> {
    let cp = c as u32;

    // Binary-search the algorithmic-name ranges first.
    let algo = ALGO_NAME_RANGES.binary_search_by(|&(start, end, _)| {
        use std::cmp::Ordering::{Equal, Greater, Less};
        if cp < start {
            Greater
        } else if cp > end {
            Less
        } else {
            Equal
        }
    });
    if let Ok(i) = algo {
        let (range_start, _, kind) = ALGO_NAME_RANGES[i];
        return algo_name(cp, range_start, kind);
    }

    // Binary-search the explicit-name table.
    CHAR_NAMES
        .binary_search_by_key(&cp, |&(c, _)| c)
        .ok()
        .map(|i| CHAR_NAMES[i].1.to_owned())
}

fn algo_name(cp: u32, range_start: u32, kind: u8) -> Option<String> {
    match kind {
        // CJK UNIFIED IDEOGRAPH-XXXX (min 4 hex digits)
        0 => Some(format!("CJK UNIFIED IDEOGRAPH-{cp:04X}")),
        // TANGUT IDEOGRAPH-XXXXX (5 hex digits)
        1 => Some(format!("TANGUT IDEOGRAPH-{cp:05X}")),
        // TANGUT COMPONENT-NNN (decimal, 1-indexed from range start)
        2 => {
            let n = cp - range_start + 1;
            Some(format!("TANGUT COMPONENT-{n:03}"))
        }
        // KHITAN SMALL SCRIPT CHARACTER-XXXX
        3 => Some(format!("KHITAN SMALL SCRIPT CHARACTER-{cp:04X}")),
        // NUSHU CHARACTER-XXXX
        4 => Some(format!("NUSHU CHARACTER-{cp:04X}")),
        // HANGUL SYLLABLE <jamo>
        5 => Some(hangul_syllable_name(cp)),
        _ => None,
    }
}

// ── Hangul syllable names (Unicode §3.12) ─────────────────────────────────────

const HANGUL_SYLLABLE_BASE: u32 = 0xAC00;
const JAMO_V_COUNT: u32 = 21;
const JAMO_T_COUNT: u32 = 28;
const JAMO_N_COUNT: u32 = JAMO_V_COUNT * JAMO_T_COUNT; // 588

static JAMO_L: &[&str] = &[
    "G", "GG", "N", "D", "DD", "R", "M", "B", "BB", "S", "SS", "", "J", "JJ", "C", "K", "T", "P",
    "H",
];
static JAMO_V: &[&str] = &[
    "A", "AE", "YA", "YAE", "EO", "E", "YEO", "YE", "O", "WA", "WAE", "OE", "YO", "U", "WEO", "WE",
    "WI", "YU", "EU", "YI", "I",
];
static JAMO_T: &[&str] = &[
    "", "G", "GG", "GS", "N", "NJ", "NH", "D", "L", "LG", "LM", "LB", "LS", "LT", "LP", "LH", "M",
    "B", "BS", "S", "SS", "NG", "J", "C", "K", "T", "P", "H",
];

fn hangul_syllable_name(cp: u32) -> String {
    let s = cp - HANGUL_SYLLABLE_BASE;
    let l = (s / JAMO_N_COUNT) as usize;
    let v = ((s % JAMO_N_COUNT) / JAMO_T_COUNT) as usize;
    let t = (s % JAMO_T_COUNT) as usize;
    format!("HANGUL SYLLABLE {}{}{}", JAMO_L[l], JAMO_V[v], JAMO_T[t])
}

// ── Range search helpers ──────────────────────────────────────────────────────

fn range_lookup<T: Copy>(table: &[(u32, u32, T)], cp: u32) -> Option<T> {
    table
        .binary_search_by(|&(start, end, _)| {
            use std::cmp::Ordering::{Equal, Greater, Less};
            if cp < start {
                Greater
            } else if cp > end {
                Less
            } else {
                Equal
            }
        })
        .ok()
        .map(|i| table[i].2)
}

fn bool_range_lookup(ranges: &[(u32, u32)], cp: u32) -> bool {
    ranges
        .binary_search_by(|&(start, end)| {
            use std::cmp::Ordering::{Equal, Greater, Less};
            if cp < start {
                Greater
            } else if cp > end {
                Less
            } else {
                Equal
            }
        })
        .is_ok()
}

// ── General Category ─────────────────────────────────────────────────────────

/// Return `(abbreviation, full_name)` for the General Category of `c`.
/// Falls back to `("Cn", "Other, Not Assigned")` for unassigned code points.
pub fn general_category_of(c: char) -> (&'static str, &'static str) {
    let idx = range_lookup(GEN_CAT_RANGES, c as u32).unwrap_or(2) as usize; // 2 = Cn
    GEN_CAT_NAMES[idx]
}

// ── Block ─────────────────────────────────────────────────────────────────────

/// Return the name of the Unicode block containing `c`, or `None`.
pub fn block_of(c: char) -> Option<&'static str> {
    range_lookup(BLOCKS, c as u32)
}

// ── Age ───────────────────────────────────────────────────────────────────────

/// Return the Unicode version string (e.g. `"1.1"`) in which `c` was first
/// assigned, or `None` if unknown.
pub fn age_of(c: char) -> Option<&'static str> {
    range_lookup(AGE_RANGES, c as u32).map(|idx| AGE_VERSIONS[idx as usize])
}

// ── Boolean properties ────────────────────────────────────────────────────────

pub fn is_alphabetic(c: char) -> bool {
    bool_range_lookup(ALPHABETIC, c as u32)
}
pub fn is_bidi_mirrored(c: char) -> bool {
    bool_range_lookup(BIDI_MIRRORED, c as u32)
}
pub fn is_case_ignorable(c: char) -> bool {
    bool_range_lookup(CASE_IGNORABLE, c as u32)
}
pub fn is_cased(c: char) -> bool {
    bool_range_lookup(CASED, c as u32)
}
pub fn is_lowercase(c: char) -> bool {
    bool_range_lookup(LOWERCASE_PROP, c as u32)
}
pub fn is_uppercase(c: char) -> bool {
    bool_range_lookup(UPPERCASE_PROP, c as u32)
}
pub fn is_white_space(c: char) -> bool {
    bool_range_lookup(WHITE_SPACE, c as u32)
}
pub fn is_noncharacter(c: char) -> bool {
    bool_range_lookup(NONCHARACTER_CODE_POINT, c as u32)
}

/// Private-use characters are those with General Category Co.
pub fn is_private_use(c: char) -> bool {
    general_category_of(c).0 == "Co"
}

pub fn is_id_start(c: char) -> bool {
    bool_range_lookup(ID_START, c as u32)
}
pub fn is_id_continue(c: char) -> bool {
    bool_range_lookup(ID_CONTINUE, c as u32)
}

// ── Script ────────────────────────────────────────────────────────────────────

/// Return the Unicode script name for `c` (e.g. `"Latin"`, `"Arabic"`), or `None`.
pub fn script_of(c: char) -> Option<&'static str> {
    range_lookup(SCRIPT_RANGES, c as u32)
}

// ── Bidi Class ────────────────────────────────────────────────────────────────

/// Return `(abbreviation, full_name)` for the Bidi_Class of `c`.
pub fn bidi_class_of(c: char) -> Option<(&'static str, &'static str)> {
    let abbr = range_lookup(BIDI_CLASS_RANGES, c as u32)?;
    Some((abbr, bidi_class_full_name(abbr)))
}

fn bidi_class_full_name(abbr: &str) -> &str {
    match abbr {
        "L" => "Left-to-Right",
        "LRE" => "Left-to-Right Embedding",
        "LRO" => "Left-to-Right Override",
        "R" => "Right-to-Left",
        "AL" => "Arabic Letter",
        "RLE" => "Right-to-Left Embedding",
        "RLO" => "Right-to-Left Override",
        "PDF" => "Pop Directional Format",
        "EN" => "European Number",
        "ES" => "European Separator",
        "ET" => "European Terminator",
        "AN" => "Arabic Number",
        "CS" => "Common Separator",
        "NSM" => "Non-Spacing Mark",
        "BN" => "Boundary Neutral",
        "B" => "Paragraph Separator",
        "S" => "Segment Separator",
        "WS" => "White Space",
        "ON" => "Other Neutral",
        "LRI" => "Left-to-Right Isolate",
        "RLI" => "Right-to-Left Isolate",
        "FSI" => "First Strong Isolate",
        "PDI" => "Pop Directional Isolate",
        _ => abbr,
    }
}

// ── Numeric Value ─────────────────────────────────────────────────────────────

/// Return the numeric value string for `c` (e.g. `"1/2"`, `"4"`), or `None`.
pub fn numeric_value_of(c: char) -> Option<&'static str> {
    NUMERIC_VALUES
        .binary_search_by_key(&(c as u32), |&(cp, _)| cp)
        .ok()
        .map(|i| NUMERIC_VALUES[i].1)
}

// ── Emoji properties ──────────────────────────────────────────────────────────

pub fn is_emoji(c: char) -> bool {
    bool_range_lookup(EMOJI, c as u32)
}
pub fn is_emoji_component(c: char) -> bool {
    bool_range_lookup(EMOJI_COMPONENT, c as u32)
}
pub fn is_emoji_modifier(c: char) -> bool {
    bool_range_lookup(EMOJI_MODIFIER, c as u32)
}
pub fn is_emoji_modifier_base(c: char) -> bool {
    bool_range_lookup(EMOJI_MODIFIER_BASE, c as u32)
}
pub fn is_emoji_presentation(c: char) -> bool {
    bool_range_lookup(EMOJI_PRESENTATION, c as u32)
}
