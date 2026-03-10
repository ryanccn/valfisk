// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

mod ucd {
    use std::{
        collections::{HashMap, HashSet},
        env,
        fs::{self, File},
        io::{self, BufRead, BufWriter, Write},
        path::{Path, PathBuf},
        process::Command,
    };

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    // ── UCD constants ──────────────────────────────────────────────────────────────

    const UNICODE_VERSION: &str = "18.0.0";
    const UCD_BASE_URL: &str = "https://unicode.org/Public/18.0.0/ucd";

    // ── Downloading ────────────────────────────────────────────────────────────────

    /// Download `name` (relative to `UCD_BASE_URL`) into `cache_dir` if not present,
    /// then declare it as a `cargo:rerun-if-changed` dependency.
    fn download_ucd_file(name: &str, cache_dir: &Path) -> Result<PathBuf> {
        let path = cache_dir.join(name);

        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            let url = format!("{UCD_BASE_URL}/{name}");
            eprintln!("cargo:warning=Downloading UCD data: {name}");

            // Try curl, fall back to wget
            let ok = try_download_curl(&url, &path).unwrap_or(false)
                || try_download_wget(&url, &path).unwrap_or(false);

            if !ok {
                return Err(format!(
                    "Failed to download {url}\n\
                     Please install curl or wget, or manually place the file at {}",
                    path.display()
                )
                .into());
            }
        }

        println!("cargo::rerun-if-changed={}", path.display());
        Ok(path)
    }

    fn try_download_curl(url: &str, dest: &Path) -> Result<bool> {
        Ok(Command::new("curl")
            .args(["-fsSL", "-o"])
            .arg(dest)
            .arg(url)
            .status()?
            .success())
    }

    fn try_download_wget(url: &str, dest: &Path) -> Result<bool> {
        Ok(Command::new("wget")
            .args(["-q", "-O"])
            .arg(dest)
            .arg(url)
            .status()?
            .success())
    }

    // ── Parsing helpers ────────────────────────────────────────────────────────────

    /// Parse a hex codepoint range like `"0041..005A"` or single point `"0041"`.
    fn parse_cp_range(s: &str) -> Result<(u32, u32)> {
        if let Some((a, b)) = s.split_once("..") {
            Ok((
                u32::from_str_radix(a.trim(), 16)?,
                u32::from_str_radix(b.trim(), 16)?,
            ))
        } else {
            let cp = u32::from_str_radix(s.trim(), 16)?;
            Ok((cp, cp))
        }
    }

    /// Strip inline comment from a UCD data line.
    fn strip_comment(line: &str) -> &str {
        if let Some(idx) = line.find('#') {
            &line[..idx]
        } else {
            line
        }
    }

    /// Parse files with the format `RANGE ; PROPERTY_VALUE  # comment`
    /// (PropList, DerivedCoreProperties, emoji-data, DerivedGeneralCategory, DerivedAge).
    /// Returns a map from property value → sorted list of (start, end) ranges.
    fn parse_named_properties(path: &Path) -> Result<HashMap<String, Vec<(u32, u32)>>> {
        let file = File::open(path)?;
        let mut props: HashMap<String, Vec<(u32, u32)>> = HashMap::new();

        for line in io::BufReader::new(file).lines() {
            let raw = line?;
            let trimmed = strip_comment(raw.trim()).trim();
            if trimmed.is_empty() {
                continue;
            }
            let (range_str, rest) = trimmed.split_once(';').ok_or("missing ';'")?;
            let prop = strip_comment(rest.trim()).trim().to_owned();
            if prop.is_empty() {
                continue;
            }
            let (start, end) = parse_cp_range(range_str)?;
            props.entry(prop).or_default().push((start, end));
        }

        for v in props.values_mut() {
            v.sort_unstable_by_key(|&(s, _)| s);
        }
        Ok(props)
    }

    /// Parse Blocks.txt: `START..END; Block Name`
    fn parse_blocks(path: &Path) -> Result<Vec<(u32, u32, String)>> {
        let file = File::open(path)?;
        let mut blocks = Vec::new();

        for line in io::BufReader::new(file).lines() {
            let raw = line?;
            let trimmed = strip_comment(raw.trim()).trim();
            if trimmed.is_empty() {
                continue;
            }
            let (range_str, name) = trimmed.split_once(';').ok_or("missing ';'")?;
            let (start, end) = parse_cp_range(range_str)?;
            blocks.push((start, end, name.trim().to_owned()));
        }

        blocks.sort_unstable_by_key(|&(s, _, _)| s);
        Ok(blocks)
    }

    /// Determine the algorithmic name type for a UnicodeData.txt range label.
    ///
    /// Returns:
    /// - 0  CJK UNIFIED IDEOGRAPH-{:04X}
    /// - 1  TANGUT IDEOGRAPH-{:05X}
    /// - 2  TANGUT COMPONENT-{decimal, 1-indexed from range start}
    /// - 3  KHITAN SMALL SCRIPT CHARACTER-{:04X}
    /// - 4  NUSHU CHARACTER-{:04X}
    /// - 5  HANGUL SYLLABLE (algorithm in src/ucd/mod.rs)
    /// - None → no formal name (surrogates, private use, etc.)
    fn range_label_to_algo_kind(label: &str) -> Option<u8> {
        if label.starts_with("CJK Ideograph") {
            Some(0)
        } else if label.starts_with("Tangut Ideograph") {
            Some(1)
        } else if label.starts_with("Tangut Component") {
            Some(2)
        } else if label.starts_with("Khitan Small Script Character") {
            Some(3)
        } else if label.starts_with("Nushu Character") {
            Some(4)
        } else if label.starts_with("Hangul Syllable") {
            Some(5)
        } else {
            None // Surrogates, Private Use → no names
        }
    }

    /// Parse UnicodeData.txt.
    ///
    /// Returns:
    /// 1. Explicit names: sorted `(codepoint, name)` pairs
    /// 2. Algorithmic name ranges: sorted `(start, end, kind)` triples
    /// 3. Bidi_Mirrored ranges: sorted and merged `(start, end)` pairs
    /// 4. Numeric values: sorted `(codepoint, value)` pairs
    #[expect(clippy::type_complexity)]
    fn parse_unicode_data(
        path: &Path,
    ) -> Result<(
        Vec<(u32, String)>,
        Vec<(u32, u32, u8)>,
        Vec<(u32, u32)>,
        Vec<(u32, String)>,
    )> {
        let file = File::open(path)?;

        let mut names: Vec<(u32, String)> = Vec::new();
        let mut algo_ranges: Vec<(u32, u32, u8)> = Vec::new();
        let mut bidi_raw: Vec<(u32, u32)> = Vec::new();
        let mut numeric_values: Vec<(u32, String)> = Vec::new();

        // Pending range: (first_cp, label, bidi_mirrored_flag)
        let mut pending: Option<(u32, String, bool)> = None;

        for line in io::BufReader::new(file).lines() {
            let raw = line?;
            let raw = raw.trim();
            if raw.is_empty() {
                continue;
            }

            let fields: Vec<&str> = raw.splitn(15, ';').collect();
            if fields.len() < 10 {
                continue;
            }

            let cp = u32::from_str_radix(fields[0].trim(), 16)?;
            let name_field = fields[1].trim();
            let bidi_mirrored = fields[9].trim() == "Y";

            if let Some(label) = name_field
                .strip_prefix('<')
                .and_then(|s| s.strip_suffix(", First>"))
            {
                pending = Some((cp, label.to_owned(), bidi_mirrored));
            } else if name_field.ends_with(", Last>") {
                if let Some((start, label, bm)) = pending.take() {
                    let end = cp;
                    if bm {
                        bidi_raw.push((start, end));
                    }
                    if let Some(kind) = range_label_to_algo_kind(&label) {
                        algo_ranges.push((start, end, kind));
                    }
                }
            } else if !name_field.is_empty() && !name_field.starts_with('<') {
                names.push((cp, name_field.to_owned()));
                if bidi_mirrored {
                    bidi_raw.push((cp, cp));
                }
                let num_val = fields[8].trim();
                if !num_val.is_empty() {
                    numeric_values.push((cp, num_val.to_owned()));
                }
            }
        }

        names.sort_unstable_by_key(|&(cp, _)| cp);
        algo_ranges.sort_unstable_by_key(|&(s, _, _)| s);
        numeric_values.sort_unstable_by_key(|&(cp, _)| cp);
        let bidi_mirrored = merge_ranges(bidi_raw);

        Ok((names, algo_ranges, bidi_mirrored, numeric_values))
    }

    // ── Range utilities ────────────────────────────────────────────────────────────

    fn merge_ranges(mut ranges: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
        if ranges.is_empty() {
            return ranges;
        }
        ranges.sort_unstable_by_key(|&(s, _)| s);
        let mut out = vec![ranges[0]];
        for (start, end) in ranges.into_iter().skip(1) {
            let last = out.last_mut().unwrap();
            if start <= last.1 + 1 {
                last.1 = last.1.max(end);
            } else {
                out.push((start, end));
            }
        }
        out
    }

    fn get_prop(map: &HashMap<String, Vec<(u32, u32)>>, name: &str) -> Vec<(u32, u32)> {
        map.get(name).cloned().map(merge_ranges).unwrap_or_default()
    }

    fn named_props_to_ranges(map: &HashMap<String, Vec<(u32, u32)>>) -> Vec<(u32, u32, String)> {
        let mut ranges: Vec<(u32, u32, String)> = map
            .iter()
            .flat_map(|(name, rs)| rs.iter().map(move |&(s, e)| (s, e, name.clone())))
            .collect();
        ranges.sort_unstable_by_key(|&(s, _, _)| s);
        ranges
    }

    // ── General Category index ─────────────────────────────────────────────────────

    /// Map a 2-letter General Category abbreviation to its index in GEN_CAT_NAMES.
    /// The table is sorted alphabetically by abbreviation.
    fn gen_cat_index(abbr: &str) -> Option<u8> {
        // Cc=0 Cf=1 Cn=2 Co=3 Cs=4
        // Ll=5 Lm=6 Lo=7 Lt=8 Lu=9
        // Mc=10 Me=11 Mn=12
        // Nd=13 Nl=14 No=15
        // Pc=16 Pd=17 Pe=18 Pf=19 Pi=20 Po=21 Ps=22
        // Sc=23 Sk=24 Sm=25 So=26
        // Zl=27 Zp=28 Zs=29
        match abbr {
            "Cc" => Some(0),
            "Cf" => Some(1),
            "Cn" => Some(2),
            "Co" => Some(3),
            "Cs" => Some(4),
            "Ll" => Some(5),
            "Lm" => Some(6),
            "Lo" => Some(7),
            "Lt" => Some(8),
            "Lu" => Some(9),
            "Mc" => Some(10),
            "Me" => Some(11),
            "Mn" => Some(12),
            "Nd" => Some(13),
            "Nl" => Some(14),
            "No" => Some(15),
            "Pc" => Some(16),
            "Pd" => Some(17),
            "Pe" => Some(18),
            "Pf" => Some(19),
            "Pi" => Some(20),
            "Po" => Some(21),
            "Ps" => Some(22),
            "Sc" => Some(23),
            "Sk" => Some(24),
            "Sm" => Some(25),
            "So" => Some(26),
            "Zl" => Some(27),
            "Zp" => Some(28),
            "Zs" => Some(29),
            _ => None,
        }
    }

    // ── Code writers ───────────────────────────────────────────────────────────────

    fn write_bool_ranges(w: &mut impl Write, name: &str, ranges: &[(u32, u32)]) -> io::Result<()> {
        writeln!(w, "pub static {name}: &[(u32, u32)] = &[")?;
        for &(s, e) in ranges {
            writeln!(w, "    (0x{s:X}, 0x{e:X}),")?;
        }
        writeln!(w, "];")?;
        writeln!(w)
    }

    fn write_str_ranges(
        w: &mut impl Write,
        name: &str,
        ranges: &[(u32, u32, String)],
    ) -> io::Result<()> {
        writeln!(w, "pub static {name}: &[(u32, u32, &str)] = &[")?;
        for (s, e, label) in ranges {
            let escaped = label.replace('\\', "\\\\").replace('"', "\\\"");
            writeln!(w, "    (0x{s:X}, 0x{e:X}, \"{escaped}\"),")?;
        }
        writeln!(w, "];")?;
        writeln!(w)
    }

    // ── Main generator ─────────────────────────────────────────────────────────────

    #[expect(clippy::too_many_lines, clippy::cast_possible_truncation)]
    pub fn generate() -> Result<()> {
        let cache_dir = Path::new(&env::var("OUT_DIR")?).join("ucd-data");
        fs::create_dir_all(&cache_dir)?;

        // Ensure all UCD data files are present
        let f_unicode_data = download_ucd_file("UnicodeData.txt", &cache_dir)?;
        let f_blocks = download_ucd_file("Blocks.txt", &cache_dir)?;
        let f_derived_age = download_ucd_file("DerivedAge.txt", &cache_dir)?;
        let f_derived_core = download_ucd_file("DerivedCoreProperties.txt", &cache_dir)?;
        let f_prop_list = download_ucd_file("PropList.txt", &cache_dir)?;
        let f_derived_gen_cat =
            download_ucd_file("extracted/DerivedGeneralCategory.txt", &cache_dir)?;
        let f_derived_bidi_class =
            download_ucd_file("extracted/DerivedBidiClass.txt", &cache_dir)?;
        let f_scripts = download_ucd_file("Scripts.txt", &cache_dir)?;
        let f_emoji_data = download_ucd_file("emoji/emoji-data.txt", &cache_dir)?;

        // Parse
        let (names, algo_ranges, bidi_mirrored, numeric_values) =
            parse_unicode_data(&f_unicode_data)?;
        let blocks = parse_blocks(&f_blocks)?;
        let ages_raw = parse_named_properties(&f_derived_age)?;
        let core_props = parse_named_properties(&f_derived_core)?;
        let prop_list = parse_named_properties(&f_prop_list)?;
        let gen_cat_data = parse_named_properties(&f_derived_gen_cat)?;
        let bidi_class_data = parse_named_properties(&f_derived_bidi_class)?;
        let scripts_data = parse_named_properties(&f_scripts)?;
        let emoji_props = parse_named_properties(&f_emoji_data)?;

        // Build general category ranges (start, end, index)
        let mut gen_cat_ranges: Vec<(u32, u32, u8)> = Vec::new();
        for (abbr, ranges) in &gen_cat_data {
            if let Some(idx) = gen_cat_index(abbr) {
                for &(s, e) in ranges {
                    gen_cat_ranges.push((s, e, idx));
                }
            }
        }
        gen_cat_ranges.sort_unstable_by_key(|&(s, _, _)| s);

        // Build age version index (sorted semantically)
        let age_version_set: HashSet<String> = ages_raw.keys().cloned().collect();
        let mut age_versions: Vec<String> = age_version_set.into_iter().collect();
        age_versions.sort_by_key(|v| {
            let mut parts = v.splitn(2, '.');
            let major: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
            let minor: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
            (major, minor)
        });
        let age_index: HashMap<&str, u8> = age_versions
            .iter()
            .enumerate()
            .map(|(i, v)| (v.as_str(), i as u8))
            .collect();

        // Build age ranges (start, end, version_index)
        let mut age_ranges: Vec<(u32, u32, u8)> = Vec::new();
        for (version, ranges) in &ages_raw {
            let idx = age_index[version.as_str()];
            for &(s, e) in ranges {
                age_ranges.push((s, e, idx));
            }
        }
        age_ranges.sort_unstable_by_key(|&(s, _, _)| s);

        // Open output
        let out_path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
            .join("src")
            .join("ucd")
            .join("data.rs");
        let mut w = BufWriter::new(File::create(&out_path)?);

        writeln!(
            w,
            "// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>\n//\n// SPDX-License-Identifier: AGPL-3.0-only"
        )?;
        writeln!(w)?;

        writeln!(
            w,
            "// Generated by `build.rs` from Unicode {UNICODE_VERSION} data. DO NOT EDIT."
        )?;
        writeln!(w)?;

        writeln!(
            w,
            "pub const UNICODE_VERSION: &str = \"{UNICODE_VERSION}\";"
        )?;
        writeln!(w)?;

        // ── General category names (index matches gen_cat_index()) ─────────────────
        writeln!(w, "pub static GEN_CAT_NAMES: &[(&str, &str)] = &[")?;
        for (abbr, full) in [
            ("Cc", "Other, Control"),
            ("Cf", "Other, Format"),
            ("Cn", "Other, Not Assigned"),
            ("Co", "Other, Private Use"),
            ("Cs", "Other, Surrogate"),
            ("Ll", "Letter, Lowercase"),
            ("Lm", "Letter, Modifier"),
            ("Lo", "Letter, Other"),
            ("Lt", "Letter, Titlecase"),
            ("Lu", "Letter, Uppercase"),
            ("Mc", "Mark, Spacing Combining"),
            ("Me", "Mark, Enclosing"),
            ("Mn", "Mark, Nonspacing"),
            ("Nd", "Number, Decimal Digit"),
            ("Nl", "Number, Letter"),
            ("No", "Number, Other"),
            ("Pc", "Punctuation, Connector"),
            ("Pd", "Punctuation, Dash"),
            ("Pe", "Punctuation, Close"),
            ("Pf", "Punctuation, Final Quote"),
            ("Pi", "Punctuation, Initial Quote"),
            ("Po", "Punctuation, Other"),
            ("Ps", "Punctuation, Open"),
            ("Sc", "Symbol, Currency"),
            ("Sk", "Symbol, Modifier"),
            ("Sm", "Symbol, Math"),
            ("So", "Symbol, Other"),
            ("Zl", "Separator, Line"),
            ("Zp", "Separator, Paragraph"),
            ("Zs", "Separator, Space"),
        ] {
            writeln!(w, "    (\"{abbr}\", \"{full}\"),")?;
        }
        writeln!(w, "];")?;
        writeln!(w)?;

        // ── General category ranges ────────────────────────────────────────────────
        writeln!(w, "pub static GEN_CAT_RANGES: &[(u32, u32, u8)] = &[")?;
        for &(s, e, idx) in &gen_cat_ranges {
            writeln!(w, "    (0x{s:X}, 0x{e:X}, {idx}),")?;
        }
        writeln!(w, "];")?;
        writeln!(w)?;

        // ── Algorithmic name ranges ────────────────────────────────────────────────
        writeln!(w, "pub static ALGO_NAME_RANGES: &[(u32, u32, u8)] = &[")?;
        for &(s, e, kind) in &algo_ranges {
            writeln!(w, "    (0x{s:X}, 0x{e:X}, {kind}),")?;
        }
        writeln!(w, "];")?;
        writeln!(w)?;

        // ── Explicit character names ───────────────────────────────────────────────
        writeln!(w, "pub static CHAR_NAMES: &[(u32, &str)] = &[")?;
        for (cp, name) in &names {
            // Names in the UCD only use ASCII printable characters, no escaping needed
            // except for backslash and double-quote (which don't appear in practice)
            writeln!(w, "    (0x{cp:X}, \"{name}\"),")?;
        }
        writeln!(w, "];")?;
        writeln!(w)?;

        // ── Blocks ────────────────────────────────────────────────────────────────
        writeln!(w, "pub static BLOCKS: &[(u32, u32, &str)] = &[")?;
        for (s, e, name) in &blocks {
            writeln!(w, "    (0x{s:X}, 0x{e:X}, \"{name}\"),")?;
        }
        writeln!(w, "];")?;
        writeln!(w)?;

        // ── Age versions ──────────────────────────────────────────────────────────
        writeln!(w, "pub static AGE_VERSIONS: &[&str] = &[")?;
        for v in &age_versions {
            writeln!(w, "    \"{v}\",")?;
        }
        writeln!(w, "];")?;
        writeln!(w)?;

        // ── Age ranges ────────────────────────────────────────────────────────────
        writeln!(w, "pub static AGE_RANGES: &[(u32, u32, u8)] = &[")?;
        for &(s, e, idx) in &age_ranges {
            writeln!(w, "    (0x{s:X}, 0x{e:X}, {idx}),")?;
        }
        writeln!(w, "];")?;
        writeln!(w)?;

        // ── Boolean properties ────────────────────────────────────────────────────
        write_bool_ranges(&mut w, "ALPHABETIC", &get_prop(&core_props, "Alphabetic"))?;
        write_bool_ranges(&mut w, "BIDI_MIRRORED", &bidi_mirrored)?;
        write_bool_ranges(
            &mut w,
            "CASE_IGNORABLE",
            &get_prop(&core_props, "Case_Ignorable"),
        )?;
        write_bool_ranges(&mut w, "CASED", &get_prop(&core_props, "Cased"))?;
        write_bool_ranges(
            &mut w,
            "LOWERCASE_PROP",
            &get_prop(&core_props, "Lowercase"),
        )?;
        write_bool_ranges(
            &mut w,
            "UPPERCASE_PROP",
            &get_prop(&core_props, "Uppercase"),
        )?;
        write_bool_ranges(&mut w, "WHITE_SPACE", &get_prop(&prop_list, "White_Space"))?;
        write_bool_ranges(
            &mut w,
            "NONCHARACTER_CODE_POINT",
            &get_prop(&prop_list, "Noncharacter_Code_Point"),
        )?;

        // ── Emoji properties ──────────────────────────────────────────────────────
        write_bool_ranges(&mut w, "EMOJI", &get_prop(&emoji_props, "Emoji"))?;
        write_bool_ranges(
            &mut w,
            "EMOJI_COMPONENT",
            &get_prop(&emoji_props, "Emoji_Component"),
        )?;
        write_bool_ranges(
            &mut w,
            "EMOJI_MODIFIER",
            &get_prop(&emoji_props, "Emoji_Modifier"),
        )?;
        write_bool_ranges(
            &mut w,
            "EMOJI_MODIFIER_BASE",
            &get_prop(&emoji_props, "Emoji_Modifier_Base"),
        )?;
        write_bool_ranges(
            &mut w,
            "EMOJI_PRESENTATION",
            &get_prop(&emoji_props, "Emoji_Presentation"),
        )?;

        // ── Script ranges ─────────────────────────────────────────────────────────
        write_str_ranges(&mut w, "SCRIPT_RANGES", &named_props_to_ranges(&scripts_data))?;

        // ── Bidi class ranges ─────────────────────────────────────────────────────
        write_str_ranges(
            &mut w,
            "BIDI_CLASS_RANGES",
            &named_props_to_ranges(&bidi_class_data),
        )?;

        // ── Numeric values ────────────────────────────────────────────────────────
        writeln!(w, "pub static NUMERIC_VALUES: &[(u32, &str)] = &[")?;
        for (cp, val) in &numeric_values {
            let escaped = val.replace('\\', "\\\\").replace('"', "\\\"");
            writeln!(w, "    (0x{cp:X}, \"{escaped}\"),")?;
        }
        writeln!(w, "];")?;
        writeln!(w)?;

        // ── Identifier properties ─────────────────────────────────────────────────
        write_bool_ranges(&mut w, "ID_START", &get_prop(&core_props, "ID_Start"))?;
        write_bool_ranges(&mut w, "ID_CONTINUE", &get_prop(&core_props, "ID_Continue"))?;

        fs::remove_dir_all(cache_dir)?;

        Ok(())
    }
}
