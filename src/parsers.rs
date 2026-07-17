use crate::publishers::KNOWN_PUBLISHERS;
use lazy_regex::{regex_captures, regex_find, regex_replace_all};

/// Represents a successfully extracted piece of data.
#[derive(Debug, PartialEq)]
pub(crate) struct Extract<'a, T> {
    /// The parsed or cleaned-up value (e.g., "Seven Seas Entertainment").
    pub(crate) value: T,
    /// The exact substring from the original string that matched (e.g., "(Seven Seas)").
    pub(crate) text: &'a str,
}

/// Extracts the extension of a file.
///
/// Uses a regex to avoid false positives from naive splits or `Path::extension()`.
/// For example:
/// ```ignore
/// use std::path::Path;
/// let p = Path::new("Youjo Senki | The Saga of Tanya the Evil Vol.26");
/// println!("{:?}", p.extension()); // Outputs: Some("26")
/// ```
///
/// Matches a dot, a letter, and 2-5 word characters at the end of the string,
/// case-insensitive.
pub(crate) fn extract_extension(s: &str) -> Option<Extract<'_, String>> {
    regex_captures!(r#"\.([a-z]\w{2,5})$"#i, s).map(|(text, ext)| Extract {
        value: ext.to_lowercase(),
        text,
    })
}

/// Extracts a known publisher using regex or substring matching
pub(crate) fn extract_publisher(s: &str) -> Option<Extract<'_, &str>> {
    // Check for Seven Seas pattern with brackets
    if let Some(text) = regex_find!(
        r#"[{(\[]\s*(Seven\sSeas\s*(?:Entertainment|Edition)?)\s*[\])}]"#i,
        s
    ) {
        return Some(Extract {
            value: "Seven Seas Entertainment",
            text,
        });
    }

    // Check for Seven Seas Siren (audiobook department) pattern with brackets
    if let Some(text) = regex_find!(r#"[{(\[]\s*(Seven\sSeas\sSiren)\s*[\])}]"#i, s) {
        return Some(Extract {
            value: "Seven Seas Siren",
            text,
        });
    }

    // Check for Kodansha pattern with brackets
    if let Some(text) = regex_find!(r#"[{(\[]\s*(Kodansha\s*(USA|Comics)?)\s*[\])}]"#i, s) {
        return Some(Extract {
            value: "Kodansha",
            text,
        });
    }

    // Check for Viz Media pattern with brackets
    if let Some(text) = regex_find!(r#"[{(\[]\s*(Viz\s*(Media)?)\s*[\])}]"#i, s) {
        return Some(Extract { value: "Viz", text });
    }

    // Check for J-Novel Club pattern with brackets
    if let Some(text) = regex_find!(r#"[{(\[]\s*(J[-\s]Novels?\sClub)\s*[\])}]"#i, s) {
        return Some(Extract {
            value: "J-Novel Club",
            text,
        });
    }

    // Check for Yen Press pattern with brackets
    if let Some(text) = regex_find!(r#"[{(\[]\s*(Yen\sPress)\s*[\])}]"#i, s) {
        return Some(Extract {
            value: "Yen Press",
            text,
        });
    }

    // Check for Yen Audio (audiobook department) pattern with brackets
    if let Some(text) = regex_find!(r#"[{(\[]\s*(Yen\sAudio)\s*[\])}]"#i, s) {
        return Some(Extract {
            value: "Yen Audio",
            text,
        });
    }

    // Check for Square Enix pattern with brackets
    if let Some(text) = regex_find!(r#"[{(\[]\s*(Square\sEnix)\s*[\])}]"#i, s) {
        return Some(Extract {
            value: "Square Enix",
            text,
        });
    }

    // --- Fallback check: Check if any known publisher name is a simple substring ---
    // If the regex patterns failed, iterate through our list of known publisher names.
    for known_publisher in KNOWN_PUBLISHERS {
        if s.contains(known_publisher) {
            return Some(Extract {
                value: known_publisher,
                text: known_publisher,
            });
        }
    }

    // If neither the regex checks nor the substring checks found a publisher, return None.
    None
}

/// Extracts a "PRE" marker.
pub(crate) fn extract_pre(s: &str) -> Option<Extract<'_, bool>> {
    regex_find!(r#"[{(\[]\s*PRE\s*[\])}]|PREPUB"#i, s).map(|text| Extract { value: true, text })
}

/// Extracts a "Digital" marker.
pub(crate) fn extract_digital(s: &str) -> Option<Extract<'_, bool>> {
    regex_find!(r#"[{(\[]\s*(digital.*?)\s*[\])}]"#i, s).map(|text| Extract { value: true, text })
}

/// Extracts a "Scan" marker.
pub(crate) fn extract_scan(s: &str) -> Option<Extract<'_, bool>> {
    regex_find!(r#"[{(\[]\s*(scan.*?)\s*[\])}]"#i, s).map(|text| Extract { value: true, text })
}

/// Extracts a "Digital-Compilation" marker.
pub(crate) fn extract_digital_compilation(s: &str) -> Option<Extract<'_, bool>> {
    regex_find!(r#"[{(\[]\s*(Digital-Compilation)\s*[\])}]"#i, s)
        .map(|text| Extract { value: true, text })
}

/// Extracts an "ED" marker.
pub(crate) fn extract_edited(s: &str) -> Option<Extract<'_, bool>> {
    regex_find!(r#"[{(\[]\s*(ed)\s*[\])}]"#i, s).map(|text| Extract { value: true, text })
}

/// Extracts the volume number.
pub(crate) fn extract_volume(s: &str) -> Option<Extract<'_, String>> {
    // First regex attempts to match patterns like "v01", "v12.5", "v01-02",
    // "001-010 as v01-02", "v01.24-04.24".
    // Regex breakdown:
    // Group 1: `(\d+-\d+\sas\s)?` - Optional leading range like "001-010 as ".
    // `v` - Matches the literal 'v'.
    // Group 2: `(\d+(\.\d+)?)` - Matches the first volume number (e.g., "01", "12.5", "01.24").
    // Group 3: `(\.\d+)?` - Optional decimal part of the first number (e.g., ".5", ".24").
    // Group 4: `(-(\d+(\.\d+)?))?` - Optional range part starting with '-'.
    // Group 5: `(\d+(\.\d+)?)` - Matches the second volume number in a range (e.g., "02", "02.25", "04.24").
    // Group 6: `(\.\d+)?` - Optional decimal part of the second number.
    // Group 7: `(\s+-[^\{\[\(\)\]\}\.]*)` - Optional whitespace separator followed by the volume title text (e.g., " - The Beginning").
    if let Some((text, _, start, _, _, end, _, _)) = regex_captures!(
        r#"(\d+-\d+\sas\s)?v(\d+(\.\d+)?)(-(\d+(\.\d+)?))?(\s+-[^\{\[\(\)\]\}\.]*)?"#i,
        &s
    ) {
        // Trim leading zeroes
        let trimmed_start = start.trim_start_matches('0');
        let trimmed_end = end.trim_start_matches('0');
        let mut vol = String::new();
        if trimmed_start.is_empty() {
            // If the trimmed start is empty (meaning the original captured 'start'
            // consisted only of zeros, e.g., "0", "00"), default the parsed volume to "0".
            vol.push('0');
        } else {
            vol.push_str(trimmed_start);
            if !trimmed_end.is_empty() {
                vol.push('-');
                vol.push_str(trimmed_end);
            }
        }

        return Some(Extract {
            value: vol,
            text: text.trim(),
        });

    // Second regex attempts to match alternate spellings like "Vol. 1", "Volume 02", "Vol 26".
    // Regex breakdown:
    // `Vol(?:ume)?` - Matches "Vol" or "Volume".
    // `(?:[\.\s]+)?` - Matches one or more periods or whitespace characters.
    // Group 1: `(\d+)` - Matches the volume number.
    // Group 2: `(\s+-[^\{\[\(\)\]\}\.]*)` - Optional whitespace separator followed by the volume title text (e.g., " - The Beginning").
    } else if let Some((text, vol, _)) = regex_captures!(
        r#"Vol(?:ume)?(?:[\.\s]+)?(\d+)(\s+-[^\{\[\(\)\]\}\.]*)?"#i,
        &s
    ) {
        let vol = vol.trim_start_matches('0');
        let parsed = if vol.is_empty() {
            // If the trimmed start is empty (meaning the original captured 'start'
            // consisted only of zeros, e.g., "0", "00"), default the parsed volume to "0".
            0.to_string()
        } else {
            vol.to_string()
        };
        return Some(Extract {
            value: parsed,
            text: text.trim(),
        });
    }
    None
}

/// Extracts the chapter number.
pub(crate) fn extract_chapter(s: &str) -> Option<Extract<'_, String>> {
    // Regex breakdown:
    // Group 1: `(\s|\bc|[,\+]\s)` - Matches the required prefix (whitespace, 'c' word boundary, ', ', or '+ ').
    // Group 2: `(\d\d+(\.\d+)?)` - Matches the first chapter number. Requires at least two digits.
    // Group 3: `(\.\d+)?` - Optional decimal part of the first number.
    // Group 4: `(-(\d\d+(\.\d+)?))?` - Optional range part starting with '-'.
    // Group 5: `(\d\d+(\.\d+)?)` - The second chapter number in a range. Requires at least two digits.
    // Group 6: `(\.\d+)?` - Optional decimal part of the second number.
    // Group 7: `(\s+-[^\{\[\(\)\]\}\.]*)` - Optional whitespace separator followed by the chapter title text (e.g., " - The Beginning").
    if let Some((text, _, start, _, _, end, _, _)) = regex_captures!(
        r#"(\s|\bc|[,\+]\s)(\d\d+(\.\d+)?)(-(\d\d+(\.\d+)?))?(\s+-[^\{\[\(\)\]\}\.]*)?"#i,
        &s
    ) {
        let trimmed_start = start.trim_start_matches('0');
        let trimmed_end = end.trim_start_matches('0');
        let mut chapter = String::new();
        if trimmed_start.is_empty() {
            // If the trimmed start is empty (meaning the original captured 'start'
            // consisted only of zeros, e.g., "0", "00"), default the parsed volume to "0".
            chapter = 0.to_string();
        } else {
            chapter.push_str(trimmed_start);
            if !trimmed_end.is_empty() {
                chapter.push('-');
                chapter.push_str(trimmed_end);
            }
        }

        return Some(Extract {
            value: chapter,
            text: text.trim(),
        });
    }
    None
}

/// Extracts the year or year range, returning only the start year in the parsed field.
pub(crate) fn extract_year(s: &str) -> Option<Extract<'_, u16>> {
    regex_captures!(r#"[{(\[]\s*(\d{4})(-\d{4})?\s*[\])}]"#i, s).and_then(
        |(text, start_year, _)| {
            start_year
                .parse::<u16>()
                .ok()
                .map(|year| Extract { value: year, text })
        },
    )
}

/// Extracts the revision marker (e.g., "(F)", "[f1]", "{r2}").
pub(crate) fn extract_revision(s: &str) -> Option<Extract<'_, u8>> {
    // The original release *without* any (f...) or (r...) tag is considered revision 1.
    // The (f...) or (r...) tag indicates subsequent revisions or "fixes" built upon revision 1.
    //
    // The revision is calculated based on the presence and value of the digit following 'f' or 'r':
    // - If no digit is present (e.g., "(f)", "[r]"), this represents the
    //   *first* revision after the original v1, hence it is revision 2.
    // - If a digit N is present (e.g., "(f1)", "[r2]", "{F3}"), this represents
    //   the Nth iteration *after* the initial "(f)" or "(r)" (v2). Therefore, the revision is N + 2.
    //
    // Examples:
    // (string without (f...) or (r...) tag) -> None (Caller should interpret it as Revision 1)
    // "(f)"   -> 2 (The first revision after v1)
    // "[f1]"  -> 3 (parsed 1 + 2)
    // "{f2}"  -> 4 (parsed 2 + 2)
    // "(F3)"  -> 5 (parsed 3 + 2)
    // "(r)"   -> 2 (The first revision after v1)
    // "[r1]"  -> 3 (parsed 1 + 2)
    // "{r2}"  -> 4 (parsed 2 + 2)
    // "(R3)"  -> 5 (parsed 3 + 2)
    //
    // https://regex101.com/r/nUDWTT/1
    regex_captures!(r#"[{(\[]\s*(?:f|r)(\d)?\s*[\])}]"#i, s).map(|(text, revision)| {
        let parsed = revision.parse::<u8>().map_or(
            2,         // If no digit present, this is the 2nd overall revision.
            |n| n + 2, // If digit N present, this is the (N + 2)th overall revision.
        );
        Extract {
            value: parsed,
            text,
        }
    })
}

/// Extracts the edition
pub(crate) fn extract_edition(s: &str) -> Option<Extract<'_, &str>> {
    // First (and simplest) case: finds any text containing "Edition" within any type of bracket.
    //
    // Example:
    // Foobar (2024) (Omnibus Edition) (Digital) (1r0n).cbz
    // |-> "Omnibus Edition"
    // https://regex101.com/r/jUz7sw/1
    if let Some((text, edition)) = regex_captures!(r#"[{(\[]([\w\s'&]*?Edition.*?)[\])}]"#i, s) {
        return Some(Extract {
            value: edition.trim(),
            text,
        });
    }

    // Second case: As a fallback, finds any text that ends with "Edition"
    // and is not necessarily enclosed in brackets. This is a broader match,
    // but the captured text consists only of word characters, whitespace,
    // single quotes, and ampersands preceding "Edition".
    //
    // Example:
    // "Tekkonkinkreet - Black & White 30th Anniversary Edition (2023) (Digital) (1r0n)"
    // |-> "Black & White 30th Anniversary Edition"
    //
    // https://regex101.com/r/umuEcH/1
    if let Some((text, edition)) = regex_captures!(r#"([\w\s'&]*?Edition)"#i, s) {
        return Some(Extract {
            value: edition.trim(),
            text: text.trim(),
        });
    }

    // Third Case: Check known editions that do not use the keyword "Edition".
    //
    // Example:
    // "The Hero-Killing Bride - Volume 02 [J-Novel Club] [Premium].epub"
    // |-> "Premium"
    //
    // https://regex101.com/r/AsSbFZ/1
    if let Some((text, edition)) = regex_captures!(r#"[{(\[](Premium)[\])}]"#i, s) {
        return Some(Extract {
            value: edition.trim(),
            text,
        });
    }

    None
}

/// Extracts the group name.
pub(crate) fn extract_group(s: &str) -> Option<Extract<'_, &str>> {
    regex_captures!(r#"[\{\[\(]([^\{\[\(\)\]\}\/\\]*)[\)\]\}]$"#i, s.trim())
        .map(|(text, group)| (text, group.trim()))
        .filter(|(_, group)| !group.is_empty())
        .map(|(text, group)| Extract { value: group, text })
}

/// Cleanup whatever's left after all the processing.
/// It's VERY important that this function is called at last,
/// after all the processing is done.
pub(crate) fn cleanup(s: &str) -> String {
    // Remove any and all terms in brackets.
    let s = regex_replace_all!(r#"[\{\[\(]([^\{\[\(\)\]\}]*)[ \)\]\}]"#i, &s, "");

    // Remove left over keywords that are certainly not part of the title.
    let s = regex_replace_all!(r#"\s*complete\s*$"#i, &s, "");

    // Some releases use the `|` or `/` character to seperate *multiple* titles
    // We only need one title, so we'll just go with the first one.
    // Examples:
    // - "Spy Kyoushitsu | Spy Classroom (2022-2023) (Digital) (1r0n)"
    // - "Attack on Titan/Shingeki no Kyojin v26 (2018) (digital-SD) [Kodansha]"
    let s = regex_replace_all!(r#"[\|\/].*"#, &s, "");

    // Remove any leading or trailing non-word characters, except for `?` and `!`.
    let s = regex_replace_all!(r#"^[^\w?!]+|[^\w?!]+$"#, &s, "");

    // Collapse multiple spaces with a single space
    let s = regex_replace_all!(r#"\s+"#, &s, " ");

    s.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("The Eminence in Shadow v01 (2021) (Digital) (1r0n).cbz", Some(Extract { value: "cbz".to_string(), text: ".cbz" }))]
    #[case("Youjo Senki | The Saga of Tanya the Evil Vol.26", None)]
    fn test_extract_extension(#[case] input: &str, #[case] expected: Option<Extract<String>>) {
        assert_eq!(extract_extension(input), expected);
    }

    #[rstest]
    #[case("Witch and Mercenary v02 [Audiobook] [Seven Seas Siren] [Stick]", Some(Extract { value: "Seven Seas Siren", text: "[Seven Seas Siren]" }))]
    #[case("The Too-Perfect Saint - Tossed Aside by My Fiancé and Sold to Another Kingdom v01-02 [Seven Seas] [nao]	", Some(Extract { value: "Seven Seas Entertainment", text: "[Seven Seas]" }))]
    #[case("Hikyouiku kara Nigetai Watashi | I Want to Escape from Princess Lessons v01 (2025) (Digital) (Seven Seas Edition) (1r0n)", Some(Extract { value: "Seven Seas Entertainment", text: "(Seven Seas Edition)"}))]
    #[case("Totto-Chan: The Little Girl at the Window [Kodansha USA] [Stick]", Some(Extract { value: "Kodansha", text: "[Kodansha USA]" }))]
    #[case("That Time I Got Reincarnated as a Slime V01-08 (danke-empire) (Kodansha Comics)	", Some(Extract { value: "Kodansha", text: "(Kodansha Comics)" }))]
    #[case("Attack on Titan/Shingeki no Kyojin v26 (2018) (digital-SD) [Kodansha]", Some(Extract { value: "Kodansha", text: "[Kodansha]" }))]
    #[case("Spy x Family - Family Portrait [VIZ Media] [Bondman]", Some(Extract { value: "Viz", text: "[VIZ Media]" }))]
    #[case("Slam Dunk - New Edition v13 (Colored Council) (Viz)", Some(Extract { value: "Viz", text: "(Viz)" }))]
    #[case("The Hero-Killing Bride - Volume 02 [J-Novel Club] [Premium].epub", Some(Extract { value: "J-Novel Club", text: "[J-Novel Club]"}))]
    #[case("The Hero-Killing Bride - Volume 02 [J Novels Club] [Premium].epub", Some(Extract { value: "J-Novel Club", text: "[J Novels Club]"}))]
    #[case("The Summer Hikaru Died v01 [Yen Press] [Stick]", Some(Extract { value: "Yen Press", text: "[Yen Press]" }))]
    #[case("Ishura v07 [Yen Audio] [Stick].m4b", Some(Extract { value: "Yen Audio", text: "[Yen Audio]" }))]
    #[case("The Healer Consort 001-010 (2025) (Digital) (Oak)", None)]
    fn test_extract_publisher(#[case] input: &str, #[case] expected: Option<Extract<&str>>) {
        assert_eq!(extract_publisher(input), expected);
    }

    #[rstest]
    #[case("Lover Boy v01 (2025) (Digital) (1r0n).cbz", Some(Extract { value: true, text: "(Digital)" }))]
    #[case("Attack on Titan v26 (2018) (digital-SD) [Kodansha].zip", Some(Extract { value: true, text: "(digital-SD)" }))]
    #[case("Dandadan 191 (2025)", None)]
    fn test_extract_digital(#[case] input: &str, #[case] expected: Option<Extract<bool>>) {
        assert_eq!(extract_digital(input), expected);
    }

    #[rstest]
    #[case("5 Centimeters per Second - One More Side - Complete [Vertical][Scans].pdf", Some(Extract { value: true, text: "[Scans]" }))]
    #[case("Alice in the Country of Diamonds - Bet on My Heart - Complete [Seven Seas][Scans_Compressed].pdf", Some(Extract { value: true, text: "[Scans_Compressed]" }))]
    fn test_extract_scan(#[case] input: &str, #[case] expected: Option<Extract<bool>>) {
        assert_eq!(extract_scan(input), expected);
    }

    #[rstest]
    #[case("Trying Out Alchemy After Being Fired as an Adventurer! 001-042 as v01-09 (Digital-Compilation) (Square Enix) (DigitalMangaFan)	", Some(Extract { value: true, text: "(Digital-Compilation)" }))]
    #[case("The Healer Consort 001-010 as v01-02 (Digital-Compilation) (Oak)", Some(Extract { value: true, text: "(Digital-Compilation)" }))]
    fn test_extract_digital_compilation(
        #[case] input: &str,
        #[case] expected: Option<Extract<bool>>,
    ) {
        assert_eq!(extract_digital_compilation(input), expected);
    }

    #[rstest]
    #[case("Smile Down the Runway v22 (2022) (Digital) (ED).cbz", Some(Extract { value: true, text: "(ED)" }))]
    #[case("Kill the Villainess 001 (2021) (Digital) (1r0n).cbz", None)]
    fn test_extract_edited(#[case] input: &str, #[case] expected: Option<Extract<bool>>) {
        assert_eq!(extract_edited(input), expected);
    }

    #[rstest]
    #[case("Natsume & Natsume v04 (2023) (Digital) (1r0n) (PRE)", Some(Extract { value: true, text: "(PRE)" }))]
    #[case("The Otome Heroine's Fight for Survival v05 Prepub.epub", Some(Extract { value: true, text: "Prepub" }))]
    fn test_extract_pre(#[case] input: &str, #[case] expected: Option<Extract<bool>>) {
        assert_eq!(extract_pre(input), expected);
    }

    #[rstest]
    #[case("The Eminence in Shadow v01 (2021) (Digital) (1r0n).cbz", Some(Extract { value: "1".to_string(), text: "v01" }))]
    #[case("The Eminence in Shadow v12.5 (2025) (Digital) (1r0n).cbz", Some(Extract { value: "12.5".to_string(), text: "v12.5" }))]
    #[case("The Death Mage v01-02 (2023-2025) (Digital) (DigitalMangaFan)", Some(Extract { value: "1-2".to_string(), text: "v01-02" }))]
    #[case("The Death Mage v01-02.25 (2023-2025) (Digital) (DigitalMangaFan)", Some(Extract { value: "1-2.25".to_string(), text: "v01-02.25" }))]
    #[case("The Banished Saint's Pilgrimage: From Dying to Thriving 001-010 as v01-02 (Digital-Compilation) (Oak)", Some(Extract { value: "1-2".to_string(), text: "001-010 as v01-02" }))]
    #[case("Programmed for Heartbreak: Sartain in Love 001-029 as v01-04 (Digital-Compilation) (Oak)", Some(Extract { value: "1-4".to_string(), text: "001-029 as v01-04" }))]
    #[case("Programmed for Heartbreak: Sartain in Love 001-029 as v01.24-04.24 (Digital-Compilation) (Oak)", Some(Extract { value: "1.24-4.24".to_string(), text: "001-029 as v01.24-04.24" }))]
    #[case("The Otome Heroine's Fight for Survival Volume 05 PREPUB [10/14]", Some(Extract { value: "5".to_string(), text: "Volume 05" }))]
    #[case("The Hero and the Sage, Reincarnated and Engaged - Volume 04 [J-Novel Club]", Some(Extract { value: "4".to_string(), text: "Volume 04" }))]
    #[case("Three Cheats from Three Goddesses: The Broke Baron’s Youngest Wants a Relaxing Life - Volume 01 [J-Novel Club]", Some(Extract { value: "1".to_string(), text: "Volume 01" }))]
    #[case("Veil - Vol 1 [We Need More Yankiis]", Some(Extract { value: "1".to_string(), text: "Vol 1" }))]
    #[case("Youjo Senki | The Saga of Tanya the Evil Vol.26", Some(Extract { value: "26".to_string(), text: "Vol.26" }))]
    #[case("fireforce_vol32.pdf", Some(Extract { value: "32".to_string(), text: "vol32" }))]
    #[case("Overlord v01 - The Undead King [Yen Press] [LuCaZ] {r3}.epub", Some(Extract { value: "1".to_string(), text: "v01 - The Undead King" }))]
    #[case("Boogiepop - Volume 01 - Boogiepop and Others.epub", Some(Extract { value: "1".to_string(), text: "Volume 01 - Boogiepop and Others" }))]
    fn test_extract_volume(#[case] input: &str, #[case] expected: Option<Extract<String>>) {
        assert_eq!(extract_volume(input), expected);
    }

    #[rstest]
    #[case("2.5 Dimensional Seduction 185.1 (2025) (Digital) (Rillant).cbz", Some(Extract { value: "185.1".to_string(), text: "185.1" }))]
    #[case("Sakamoto Days 210 (2025) (Digital) (Rillant).cbz", Some(Extract { value: "210".to_string(), text: "210" }))]
    #[case("I'm a Curse Crafter, and I Don't Need an S-Rank Party! 042.2 (2025) (Digital) (Valdearg).cbz", Some(Extract { value: "42.2".to_string(), text: "042.2" }))]
    #[case("The Case Study of Vanitas 063 (2024) (Digital) (LuCaZ).cbz", Some(Extract { value: "63".to_string(), text: "063" }))]
    #[case("Hyeonjung's Residence c57 (Void).cbz", Some(Extract { value: "57".to_string(), text: "c57" }))]
    #[case("The Crow's Prince c095 - Season 2 Finale (2022) (Digital) (Dalte).cbz", Some(Extract { value: "95".to_string(), text: "c095 - Season 2 Finale" }))]
    #[case("They ridiculed me for my luckless job, but it's not actually that bad 002 - Of Course it's Weird! (2022) (Digital) (AntsyLich)", Some(Extract { value: "2".to_string(), text: "002 - Of Course it's Weird!" }))]
    #[case("Edens Zero v01-31, 276-293 (2018-2025) (Digital) (danke-Empire, DeadMan, SlikkyOak)", Some(Extract { value: "276-293".to_string(), text: ", 276-293" }))]
    #[case("Wistoria - Wand and Sword v01-08 + 033-051 (2022-2025) (Digital) (1r0n)", Some(Extract { value: "33-51".to_string(), text: "+ 033-051" }))]
    #[case("Merin the Mermaid - 00 - Prologue (Digital) (Cobalt001)", Some(Extract { value: "0".to_string(), text: "00 - Prologue" }))]
    fn test_extract_chapter(#[case] input: &str, #[case] expected: Option<Extract<String>>) {
        assert_eq!(extract_chapter(input), expected);
    }

    #[rstest]
    #[case("[Unpaid Ferryman] Gamaran: Shura v01-31 (2022-2025) (Digital) (danke-Empire, Kaos, Rillant)", Some(Extract { value: 2022, text: "(2022-2025)" }))]
    #[case("The Healer Consort 001-010 (2025) (Digital) (Oak)", Some(Extract { value: 2025, text: "(2025)" }))]
    #[case("The Eminence in Shadow v01-12 (2021-2025) (Digital) (1r0n)", Some(Extract { value: 2021, text: "(2021-2025)" }))]
    fn test_extract_year(#[case] input: &str, #[case] expected: Option<Extract<u16>>) {
        assert_eq!(extract_year(input), expected);
    }

    #[rstest]
    #[case("One-Punch Man 193 (2024) (Digital) (Rillant) (f).cbz", Some(Extract { value: 2, text: "(f)" }))]
    #[case("One-Punch Man 193 (2024) (Digital) (Rillant) {f2}.cbz", Some(Extract { value: 4, text: "{f2}" }))]
    #[case("The Beginning After the End, Vol. 11 [PZG] {r2}.m4b", Some(Extract { value: 4, text: "{r2}" }))]
    #[case("The Healer Consort 001-010 (2025) (Digital) (Oak)", None)]
    fn test_extract_revision(#[case] input: &str, #[case] expected: Option<Extract<u8>>) {
        assert_eq!(extract_revision(input), expected);
    }

    #[rstest]
    #[case("Uzumaki (2018) (Digital) (Deluxe Edition 3-in-1) (Mr. Kimiko-Teikō) (ED).cbz", Some(Extract { value: "Deluxe Edition 3-in-1", text: "(Deluxe Edition 3-in-1)" }))]
    #[case("Yokohama Kaidashi Kikou - Deluxe Edition (2022-2024) (Digital) (1r0n).cbz", Some(Extract { value: "Deluxe Edition", text: "Deluxe Edition" }))]
    #[case("Adults' Picture Book - New Edition v01-02 (2024) (Digital) (LuCaZ).cbz", Some(Extract { value: "New Edition", text: "New Edition" }))]
    #[case("Gravitation - Collector's Edition v01 (2024) (Digital) (LuCaZ).cbz", Some(Extract { value: "Collector's Edition", text: "Collector's Edition" }))]
    #[case("My Name Is Shingo - The Perfect Edition v01-02 (2024) (Digital) (1r0n).cbz", Some(Extract { value: "The Perfect Edition", text: "The Perfect Edition" }))]
    #[case("86--EIGHTY-SIX - Operation High School (2024) (Omnibus Edition) (Digital) (1r0n).cbz", Some(Extract { value: "Omnibus Edition", text: "(Omnibus Edition)" }))]
    #[case("86--EIGHTY-SIX - Operation High School (2024) (Collector's Edition) (Digital) (1r0n).cbz", Some(Extract { value: "Collector's Edition", text: "(Collector's Edition)" }))]
    #[case("Zatch Bell! Revamped Edition v01 (2018 E-Book) (Zatch Bell Makai Scanlations)", Some(Extract { value: "Revamped Edition", text: "Revamped Edition" }))]
    #[case("Hellsing v01-03 (2023-2024) (Second Edition) (Digital) (LuCaZ)", Some(Extract { value: "Second Edition", text: "(Second Edition)" }))]
    #[case("Magic Knight Rayearth 2 {25th Anniversary Edition} (2020) (Digital) (XRA-Empire)", Some(Extract { value: "25th Anniversary Edition", text: "{25th Anniversary Edition}" }))]
    #[case("Gunsmith Cats - Revised Edition (2007) (Digital) (XRA-Empire)", Some(Extract { value: "Revised Edition", text: "Revised Edition" }))]
    #[case("Tekkonkinkreet - Black & White 30th Anniversary Edition (2023) (Digital) (1r0n)", Some(Extract { value: "Black & White 30th Anniversary Edition", text: "Black & White 30th Anniversary Edition" }))]
    #[case("Tekkonkinkreet - (Black & White 30th Anniversary Edition) (2023) (Digital) (1r0n)", Some(Extract { value: "Black & White 30th Anniversary Edition", text: "(Black & White 30th Anniversary Edition)" }))]
    fn test_extract_edition(#[case] input: &str, #[case] expected: Option<Extract<&str>>) {
        assert_eq!(extract_edition(input), expected);
    }

    #[rstest]
    #[case("Lover Boy v01 (2025) (Digital) (1r0n)", Some(Extract { value: "1r0n", text: "(1r0n)" }))]
    #[case("Witch and Mercenary v02 [Audiobook] [Seven Seas Siren] [Stick]", Some(Extract { value: "Stick", text: "[Stick]" }))]
    fn test_extract_group(#[case] input: &str, #[case] expected: Option<Extract<&str>>) {
        assert_eq!(extract_group(input), expected);
    }

    #[rstest]
    #[case("Tekkonkinkreet - (Black & White 30th Anniversary Edition) (2023) (Digital) (1r0n)", "Tekkonkinkreet".to_string())]
    #[case("Youjo Senki | The Saga of Tanya the Evil Vol.26", "Youjo Senki".to_string())]
    #[case("The Beginning After the End,", "The Beginning After the End".to_string())]
    fn test_cleanup(#[case] input: &str, #[case] expected: String) {
        assert_eq!(cleanup(input), expected);
    }
}
