use crate::publishers::KNOWN_PUBLISHERS;
use lazy_regex::{regex_captures, regex_find, regex_replace_all};

/// Represents a successfully extracted piece of data.
#[derive(Debug, PartialEq)]
pub(crate) struct Match<'a, T> {
    /// The parsed or cleaned-up value (e.g., "Seven Seas Entertainment").
    pub(crate) parsed: T,
    /// The exact substring from the original string that matched (e.g., "(Seven Seas)").
    pub(crate) raw: &'a str,
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
pub(crate) fn extract_extension(s: &str) -> Option<Match<'_, String>> {
    if let Some((raw, ext)) = regex_captures!(r#"\.([a-z]\w{2,5})$"#i, s) {
        Some(Match {
            parsed: ext.to_lowercase(),
            raw,
        })
    } else {
        None
    }
}

/// Extracts a known publisher using regex or substring matching, returning
/// both the raw matched string and the parsed string slice.
pub(crate) fn extract_publisher(s: &str) -> Option<Match<'_, &str>> {
    // Check for Seven Seas pattern with brackets
    if let Some(raw) = regex_find!(
        r#"(\(|\[|\{)\s*(Seven\sSeas\s*(Siren|Entertainment|Edition)?)\s*(\}|\]|\))"#i,
        s
    ) {
        return Some(Match {
            parsed: "Seven Seas Entertainment",
            raw,
        });
    }

    // Check for Kodansha pattern with brackets
    if let Some(raw) = regex_find!(
        r#"(\(|\[|\{)\s*(Kodansha\s*(USA|Comics)?)\s*(\}|\]|\))"#i,
        s
    ) {
        return Some(Match {
            parsed: "Kodansha",
            raw,
        });
    }

    // Check for Viz Media pattern with brackets
    if let Some(raw) = regex_find!(r#"(\(|\[|\{)\s*(Viz\s*(Media)?)\s*(\}|\]|\))"#i, s) {
        return Some(Match { parsed: "Viz", raw });
    }

    // Check for J-Novel Club pattern with brackets
    if let Some(raw) = regex_find!(r#"(\(|\[|\{)\s*(J[-\s]Novels?\sClub)\s*(\}|\]|\))"#i, s) {
        return Some(Match {
            parsed: "J-Novel Club",
            raw,
        });
    }

    // Check for Yen Press pattern with brackets
    if let Some(raw) = regex_find!(r#"(\(|\[|\{)\s*(Yen\sPress)\s*(\}|\]|\))"#i, s) {
        return Some(Match {
            parsed: "Yen Press",
            raw,
        });
    }

    // Check for Square Enix pattern with brackets
    if let Some(raw) = regex_find!(r#"(\(|\[|\{)\s*(Square\sEnix)\s*(\}|\]|\))"#i, s) {
        return Some(Match {
            parsed: "Square Enix",
            raw,
        });
    }

    // --- Fallback check: Check if any known publisher name is a simple substring ---
    // If the regex patterns failed, iterate through our list of known publisher names.
    for known_publisher in KNOWN_PUBLISHERS {
        if s.contains(known_publisher) {
            return Some(Match {
                parsed: known_publisher,
                raw: known_publisher,
            });
        }
    }

    // If neither the regex checks nor the substring checks found a publisher, return None.
    None
}

/// Extracts a "PRE" marker.
pub(crate) fn extract_pre(s: &str) -> Option<Match<'_, bool>> {
    regex_find!(r#"[\(\[\{]\s*PRE\s*[\}\]\)]|PREPUB"#i, s).map(|raw| Match { parsed: true, raw })
}

/// Extracts a "Digital" marker.
pub(crate) fn extract_digital(s: &str) -> Option<Match<'_, bool>> {
    regex_find!(r#"(\(|\[|\{)\s*(digital)\s*(\}|\]|\))"#i, s)
        .map(|raw| Match { parsed: true, raw })
}

/// Extracts a "Digital-Compilation" marker.
pub(crate) fn extract_digital_compilation(s: &str) -> Option<Match<'_, bool>> {
    regex_find!(r#"(\(|\[|\{)\s*(Digital-Compilation)\s*(\}|\]|\))"#i, s)
        .map(|raw| Match { parsed: true, raw })
}

/// Extracts an "ED" marker.
pub(crate) fn extract_edited(s: &str) -> Option<Match<'_, bool>> {
    regex_find!(r#"(\(|\[|\{)\s*(ed)\s*(\}|\]|\))"#i, s).map(|raw| Match { parsed: true, raw })
}

/// Extracts the volume number.
pub(crate) fn extract_volume(s: &str) -> Option<Match<'_, String>> {
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
    // The function uses the full match (Group 0) for `raw`, Group 2 for `start`, and Group 5 for `end`.
    // https://regex101.com/r/X9IJaF/1
    if let Some((raw, _, start, _, _, end, _)) =
        regex_captures!(r#"(\d+-\d+\sas\s)?v(\d+(\.\d+)?)(-(\d+(\.\d+)?))?"#i, &s)
    {
        // Trim leading zeroes
        let trimmed_start = start.trim_start_matches("0");
        let trimmed_end = end.trim_start_matches("0");
        let mut vol = String::new();
        if !trimmed_start.is_empty() {
            vol.push_str(trimmed_start);
            if !trimmed_end.is_empty() {
                vol.push('-');
                vol.push_str(trimmed_end);
            }
        } else {
            // If the trimmed start is empty (meaning the original captured 'start'
            // consisted only of zeros, e.g., "0", "00"), default the parsed volume to "0".
            vol = 0.to_string();
        }

        return Some(Match { parsed: vol, raw });

    // Second regex attempts to match alternate spellings like "Vol. 1", "Volume 02", "Vol 26".
    // Regex breakdown:
    // `Vol(?:ume)?` - Matches "Vol" or "Volume".
    // `(?:[\.\s]+)?` - Matches one or more periods or whitespace characters.
    // Group 1: `(\d+)` - Matches the volume number.
    // The function uses the full match (Group 0) for `raw` and Group 1 for `vol`.
    // https://regex101.com/r/HUAeaX/1
    } else if let Some((raw, vol)) = regex_captures!(r#"Vol(?:ume)?(?:[\.\s]+)?(\d+)"#i, &s) {
        let vol = vol.trim_start_matches("0");
        let parsed = if vol.is_empty() {
            // If the trimmed start is empty (meaning the original captured 'start'
            // consisted only of zeros, e.g., "0", "00"), default the parsed volume to "0".
            0.to_string()
        } else {
            vol.to_string()
        };
        return Some(Match { parsed, raw });
    }
    None
}

/// Extracts the chapter number.
pub(crate) fn extract_chapter(s: &str) -> Option<Match<'_, String>> {
    // Regex breakdown:
    // Group 1: `(\s|\bc|[,\+]\s)` - Matches the required prefix (whitespace, 'c' word boundary, ', ', or '+ ').
    // Group 2: `(\d\d+(\.\d+)?)` - Matches the first chapter number. Requires at least two digits.
    // Group 3: `(\.\d+)?` - Optional decimal part of the first number.
    // Group 4: `(-(\d\d+(\.\d+)?))?` - Optional range part starting with '-'.
    // Group 5: `(\d\d+(\.\d+)?)` - The second chapter number in a range. Requires at least two digits.
    // Group 6: `(\.\d+)?` - Optional decimal part of the second number.
    // `\s` - Matches the space after the number or range.
    // Group 7: `(-[^\{\[\(\)\]\}]*)?` - Optional chapter title.
    // The function uses the full match (Group 0) for `raw`, Group 2 for `start`, and Group 5 for `end`.
    // https://regex101.com/r/uzn3Xh/1
    if let Some((raw, _, start, _, _, end, _, _)) = regex_captures!(
        r#"(\s|\bc|[,\+]\s)(\d\d+(\.\d+)?)(-(\d\d+(\.\d+)?))?\s(-[^\{\[\(\)\]\}]*)?"#i,
        &s
    ) {
        let trimmed_start = start.trim_start_matches("0");
        let trimmed_end = end.trim_start_matches("0");
        let mut chapter = String::new();
        if !trimmed_start.is_empty() {
            chapter.push_str(trimmed_start);
            if !trimmed_end.is_empty() {
                chapter.push('-');
                chapter.push_str(trimmed_end);
            }
        } else {
            // If the trimmed start is empty (meaning the original captured 'start'
            // consisted only of zeros, e.g., "0", "00"), default the parsed volume to "0".
            chapter = 0.to_string();
        }

        return Some(Match {
            parsed: chapter,
            raw: raw.trim(),
        });
    }
    None
}

/// Extracts the year or year range, returning only the start year in the parsed field.
pub(crate) fn extract_year(s: &str) -> Option<Match<'_, u16>> {
    if let Some((raw, _, start_year, _, _)) =
        regex_captures!(r#"(\(|\[|\{)\s*(\d{4})(-\d{4})?\s*(\}|\]|\))"#i, s)
    {
        // UNWRAP SAFETY: unwrap is infallible here,
        // because the regex captures exactly 4 digits
        // which will always be within u16.
        let year = start_year.parse::<u16>().unwrap();
        Some(Match { parsed: year, raw })
    } else {
        None
    }
}

/// Extracts the revision marker (e.g., "(F)", "[f1]").
pub(crate) fn extract_revision(s: &str) -> Option<Match<'_, u8>> {
    // The original release *without* any (f...) tag is considered revision 1.
    // The (f...) tag indicates subsequent revisions or "fixes" built upon revision 1.
    //
    // The revision is calculated based on the presence and value of the digit following 'f':
    // - If no digit is present (e.g., "(f)"), this represents the
    //   *first* revision after the original v1, hence it is revision 2.
    // - If a digit N is present (e.g., "(f1)", "[f2]", "{f3}"), this represents
    //   the Nth iteration *after* the initial "(f)" (v2). Therefore, the revision is N + 2.
    //
    // Examples:
    // (string without (f...) tag) -> None (Caller interprets as Revision 1)
    // "(f)"   -> 2 (The first revision after v1)
    // "[f1]"  -> 3 (parsed 1 + 2)
    // "{f2}"  -> 4 (parsed 2 + 2)
    // "(F3)"  -> 5 (parsed 3 + 2)
    if let Some((raw, _, revision, _)) = regex_captures!(r#"(\(|\[|\{)\s*f(\d)?\s*(\}|\]|\))"#i, s)
    {
        let rev: u8 = if revision.is_empty() {
            // Case: "(f)", "[f]", "{f}" - no digit. This is the 2nd overall revision.
            2
        } else {
            // Case: "(fN)", "[fN]", "{fN}" - digit N is present. This is the (N + 2)th overall revision.

            // UNWRAP SAFETY: The regex `(\d)?` guarantees that if revision is not empty, it contains exactly
            // one digit. Therefore, parse::<u8>() is guaranteed to succeed for valid inputs
            // matching the regex, making the unwrap safe from parse errors.
            let n = revision.parse::<u8>().unwrap();
            n + 2
        };
        Some(Match { parsed: rev, raw })
    } else {
        None
    }
}

/// Extracts the edition
pub(crate) fn extract_edition(s: &str) -> Option<Match<'_, &str>> {
    // First (and simplest) case: finds any text containing "Edition" within any type of bracket.
    //
    // Example:
    // Foobar (2024) (Omnibus Edition) (Digital) (1r0n).cbz
    // |-> "Omnibus Edition"
    // https://regex101.com/r/VYmT3A/1
    if let Some((raw, edition)) = regex_captures!(r#"[\{\[\(]([\w\s'&]*?Edition.*?)[\)\]\}]"#i, s)
    {
        return Some(Match {
            parsed: edition.trim(),
            raw,
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
    // https://regex101.com/r/YXixDi/1
    if let Some((raw, edition)) = regex_captures!(r#"([\w\s'&]*?Edition)"#i, s) {
        return Some(Match {
            parsed: edition.trim(),
            raw: raw.trim(),
        });
    }

    // Third Case: Check known editions that do not use the keyword "Edition".
    //
    // Example:
    // "The Hero-Killing Bride - Volume 02 [J-Novel Club] [Premium].epub"
    // |-> "Premium"
    //
    // https://regex101.com/r/XzruAL/1
    if let Some((raw, edition)) = regex_captures!(r#"[\{\[\(](Premium)[\)\]\}]"#i, s) {
        return Some(Match {
            parsed: edition.trim(),
            raw,
        });
    }

    None
}

/// Extracts the group name.
pub(crate) fn extract_group(s: &str) -> Option<Match<'_, &str>> {
    if let Some((raw, group)) =
        regex_captures!(r#"[\{\[\(]([^\{\[\(\)\]\}\/\\]*)[\)\]\}]$"#i, s.trim())
    {
        let parsed = group.trim();
        Some(Match { parsed, raw })
    } else {
        None
    }
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

    // Collapse multiple spaces with a single space
    let s = regex_replace_all!(r#"\s+"#, &s, " ");

    s.trim().trim_matches('-').trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("The Eminence in Shadow v01 (2021) (Digital) (1r0n).cbz", Some(Match { parsed: "cbz".to_string(), raw: ".cbz" }))]
    #[case("Youjo Senki | The Saga of Tanya the Evil Vol.26", None)]
    fn test_extract_extension(#[case] input: &str, #[case] expected: Option<Match<String>>) {
        assert_eq!(extract_extension(input), expected);
    }

    #[rstest]
    #[case("Witch and Mercenary v02 [Audiobook] [Seven Seas Siren] [Stick]", Some(Match { parsed: "Seven Seas Entertainment", raw: "[Seven Seas Siren]" }))]
    #[case("The Too-Perfect Saint - Tossed Aside by My Fiancé and Sold to Another Kingdom v01-02 [Seven Seas] [nao]	", Some(Match { parsed: "Seven Seas Entertainment", raw: "[Seven Seas]" }))]
    #[case("Hikyouiku kara Nigetai Watashi | I Want to Escape from Princess Lessons v01 (2025) (Digital) (Seven Seas Edition) (1r0n)", Some(Match { parsed: "Seven Seas Entertainment", raw: "(Seven Seas Edition)"}))]
    #[case("Totto-Chan: The Little Girl at the Window [Kodansha USA] [Stick]", Some(Match { parsed: "Kodansha", raw: "[Kodansha USA]" }))]
    #[case("That Time I Got Reincarnated as a Slime V01-08 (danke-empire) (Kodansha Comics)	", Some(Match { parsed: "Kodansha", raw: "(Kodansha Comics)" }))]
    #[case("Attack on Titan/Shingeki no Kyojin v26 (2018) (digital-SD) [Kodansha]", Some(Match { parsed: "Kodansha", raw: "[Kodansha]" }))]
    #[case("Spy x Family - Family Portrait [VIZ Media] [Bondman]", Some(Match { parsed: "Viz", raw: "[VIZ Media]" }))]
    #[case("Slam Dunk - New Edition v13 (Colored Council) (Viz)", Some(Match { parsed: "Viz", raw: "(Viz)" }))]
    #[case("The Hero-Killing Bride - Volume 02 [J-Novel Club] [Premium].epub", Some(Match { parsed: "J-Novel Club", raw: "[J-Novel Club]"}))]
    #[case("The Hero-Killing Bride - Volume 02 [J Novels Club] [Premium].epub", Some(Match { parsed: "J-Novel Club", raw: "[J Novels Club]"}))]
    #[case("The Summer Hikaru Died v01 [Yen Press] [Stick]", Some(Match { parsed: "Yen Press", raw: "[Yen Press]" }))]
    #[case("The Healer Consort 001-010 (2025) (Digital) (Oak)", None)]
    fn test_extract_publisher(#[case] input: &str, #[case] expected: Option<Match<&str>>) {
        assert_eq!(extract_publisher(input), expected);
    }

    #[rstest]
    #[case("Lover Boy v01 (2025) (Digital) (1r0n).cbz", Some(Match { parsed: true, raw: "(Digital)" }))]
    #[case("Dandadan 191 (2025)", None)]
    fn test_extract_digital(#[case] input: &str, #[case] expected: Option<Match<bool>>) {
        assert_eq!(extract_digital(input), expected);
    }

    #[rstest]
    #[case("Trying Out Alchemy After Being Fired as an Adventurer! 001-042 as v01-09 (Digital-Compilation) (Square Enix) (DigitalMangaFan)	", Some(Match { parsed: true, raw: "(Digital-Compilation)" }))]
    #[case("The Healer Consort 001-010 as v01-02 (Digital-Compilation) (Oak)", Some(Match { parsed: true, raw: "(Digital-Compilation)" }))]
    fn test_extract_digital_compilation(
        #[case] input: &str,
        #[case] expected: Option<Match<bool>>,
    ) {
        assert_eq!(extract_digital_compilation(input), expected);
    }

    #[rstest]
    #[case("Smile Down the Runway v22 (2022) (Digital) (ED).cbz", Some(Match { parsed: true, raw: "(ED)" }))]
    #[case("Kill the Villainess 001 (2021) (Digital) (1r0n).cbz", None)]
    fn test_extract_edited(#[case] input: &str, #[case] expected: Option<Match<bool>>) {
        assert_eq!(extract_edited(input), expected);
    }

    #[rstest]
    #[case("Natsume & Natsume v04 (2023) (Digital) (1r0n) (PRE)", Some(Match { parsed: true, raw: "(PRE)" }))]
    #[case("The Otome Heroine's Fight for Survival v05 Prepub.epub", Some(Match { parsed: true, raw: "Prepub" }))]
    fn test_extract_pre(#[case] input: &str, #[case] expected: Option<Match<bool>>) {
        assert_eq!(extract_pre(input), expected);
    }

    #[rstest]
    #[case("The Eminence in Shadow v01 (2021) (Digital) (1r0n).cbz", Some(Match { parsed: "1".to_string(), raw: "v01" }))]
    #[case("The Eminence in Shadow v12.5 (2025) (Digital) (1r0n).cbz", Some(Match { parsed: "12.5".to_string(), raw: "v12.5" }))]
    #[case("The Death Mage v01-02 (2023-2025) (Digital) (DigitalMangaFan)", Some(Match { parsed: "1-2".to_string(), raw: "v01-02" }))]
    #[case("The Death Mage v01-02.25 (2023-2025) (Digital) (DigitalMangaFan)", Some(Match { parsed: "1-2.25".to_string(), raw: "v01-02.25" }))]
    #[case("The Banished Saint's Pilgrimage: From Dying to Thriving 001-010 as v01-02 (Digital-Compilation) (Oak)", Some(Match { parsed: "1-2".to_string(), raw: "001-010 as v01-02" }))]
    #[case("Programmed for Heartbreak: Sartain in Love 001-029 as v01-04 (Digital-Compilation) (Oak)", Some(Match { parsed: "1-4".to_string(), raw: "001-029 as v01-04" }))]
    #[case("Programmed for Heartbreak: Sartain in Love 001-029 as v01.24-04.24 (Digital-Compilation) (Oak)", Some(Match { parsed: "1.24-4.24".to_string(), raw: "001-029 as v01.24-04.24" }))]
    #[case("The Otome Heroine's Fight for Survival Volume 05 PREPUB [10/14]", Some(Match { parsed: "5".to_string(), raw: "Volume 05" }))]
    #[case("The Hero and the Sage, Reincarnated and Engaged - Volume 04 [J-Novel Club]", Some(Match { parsed: "4".to_string(), raw: "Volume 04" }))]
    #[case("Three Cheats from Three Goddesses: The Broke Baron’s Youngest Wants a Relaxing Life - Volume 01 [J-Novel Club]", Some(Match { parsed: "1".to_string(), raw: "Volume 01" }))]
    #[case("Veil - Vol 1 [We Need More Yankiis]", Some(Match { parsed: "1".to_string(), raw: "Vol 1" }))]
    #[case("Youjo Senki | The Saga of Tanya the Evil Vol.26", Some(Match { parsed: "26".to_string(), raw: "Vol.26" }))]
    #[case("fireforce_vol32.pdf", Some(Match { parsed: "32".to_string(), raw: "vol32" }))]
    fn test_extract_volume(#[case] input: &str, #[case] expected: Option<Match<String>>) {
        assert_eq!(extract_volume(input), expected);
    }

    #[rstest]
    #[case("2.5 Dimensional Seduction 185.1 (2025) (Digital) (Rillant).cbz", Some(Match { parsed: "185.1".to_string(), raw: "185.1" }))]
    #[case("Sakamoto Days 210 (2025) (Digital) (Rillant).cbz", Some(Match { parsed: "210".to_string(), raw: "210" }))]
    #[case("I'm a Curse Crafter, and I Don't Need an S-Rank Party! 042.2 (2025) (Digital) (Valdearg).cbz", Some(Match { parsed: "42.2".to_string(), raw: "042.2" }))]
    #[case("The Case Study of Vanitas 063 (2024) (Digital) (LuCaZ).cbz", Some(Match { parsed: "63".to_string(), raw: "063" }))]
    #[case("Hyeonjung's Residence c57 (Void).cbz", Some(Match { parsed: "57".to_string(), raw: "c57" }))]
    #[case("The Crow's Prince c095 - Season 2 Finale (2022) (Digital) (Dalte).cbz", Some(Match { parsed: "95".to_string(), raw: "c095 - Season 2 Finale" }))]
    #[case("They ridiculed me for my luckless job, but it's not actually that bad 002 - Of Course it's Weird! (2022) (Digital) (AntsyLich)", Some(Match { parsed: "2".to_string(), raw: "002 - Of Course it's Weird!" }))]
    #[case("Edens Zero v01-31, 276-293 (2018-2025) (Digital) (danke-Empire, DeadMan, SlikkyOak)", Some(Match { parsed: "276-293".to_string(), raw: ", 276-293" }))]
    #[case("Wistoria - Wand and Sword v01-08 + 033-051 (2022-2025) (Digital) (1r0n)", Some(Match { parsed: "33-51".to_string(), raw: "+ 033-051" }))]
    #[case("Merin the Mermaid - 00 - Prologue (Digital) (Cobalt001)", Some(Match { parsed: "0".to_string(), raw: "00 - Prologue" }))]
    fn test_extract_chapter(#[case] input: &str, #[case] expected: Option<Match<String>>) {
        assert_eq!(extract_chapter(input), expected);
    }

    #[rstest]
    #[case("[Unpaid Ferryman] Gamaran: Shura v01-31 (2022-2025) (Digital) (danke-Empire, Kaos, Rillant)", Some(Match { parsed: 2022, raw: "(2022-2025)" }))]
    #[case("The Healer Consort 001-010 (2025) (Digital) (Oak)", Some(Match { parsed: 2025, raw: "(2025)" }))]
    #[case("The Eminence in Shadow v01-12 (2021-2025) (Digital) (1r0n)", Some(Match { parsed: 2021, raw: "(2021-2025)" }))]
    fn test_extract_year(#[case] input: &str, #[case] expected: Option<Match<u16>>) {
        assert_eq!(extract_year(input), expected);
    }

    #[rstest]
    #[case("One-Punch Man 193 (2024) (Digital) (Rillant) (f).cbz", Some(Match { parsed: 2, raw: "(f)" }))]
    #[case("One-Punch Man 193 (2024) (Digital) (Rillant) {f2}.cbz", Some(Match { parsed: 4, raw: "{f2}" }))]
    #[case("The Healer Consort 001-010 (2025) (Digital) (Oak)", None)]
    fn test_extract_revision(#[case] input: &str, #[case] expected: Option<Match<u8>>) {
        assert_eq!(extract_revision(input), expected);
    }

    #[rstest]
    #[case("Uzumaki (2018) (Digital) (Deluxe Edition 3-in-1) (Mr. Kimiko-Teikō) (ED).cbz", Some(Match { parsed: "Deluxe Edition 3-in-1", raw: "(Deluxe Edition 3-in-1)" }))]
    #[case("Yokohama Kaidashi Kikou - Deluxe Edition (2022-2024) (Digital) (1r0n).cbz", Some(Match { parsed: "Deluxe Edition", raw: "Deluxe Edition" }))]
    #[case("Adults' Picture Book - New Edition v01-02 (2024) (Digital) (LuCaZ).cbz", Some(Match { parsed: "New Edition", raw: "New Edition" }))]
    #[case("Gravitation - Collector's Edition v01 (2024) (Digital) (LuCaZ).cbz", Some(Match { parsed: "Collector's Edition", raw: "Collector's Edition" }))]
    #[case("My Name Is Shingo - The Perfect Edition v01-02 (2024) (Digital) (1r0n).cbz", Some(Match { parsed: "The Perfect Edition", raw: "The Perfect Edition" }))]
    #[case("86--EIGHTY-SIX - Operation High School (2024) (Omnibus Edition) (Digital) (1r0n).cbz", Some(Match { parsed: "Omnibus Edition", raw: "(Omnibus Edition)" }))]
    #[case("86--EIGHTY-SIX - Operation High School (2024) (Collector's Edition) (Digital) (1r0n).cbz", Some(Match { parsed: "Collector's Edition", raw: "(Collector's Edition)" }))]
    #[case("Zatch Bell! Revamped Edition v01 (2018 E-Book) (Zatch Bell Makai Scanlations)", Some(Match { parsed: "Revamped Edition", raw: "Revamped Edition" }))]
    #[case("Hellsing v01-03 (2023-2024) (Second Edition) (Digital) (LuCaZ)", Some(Match { parsed: "Second Edition", raw: "(Second Edition)" }))]
    #[case("Magic Knight Rayearth 2 {25th Anniversary Edition} (2020) (Digital) (XRA-Empire)", Some(Match { parsed: "25th Anniversary Edition", raw: "{25th Anniversary Edition}" }))]
    #[case("Gunsmith Cats - Revised Edition (2007) (Digital) (XRA-Empire)", Some(Match { parsed: "Revised Edition", raw: "Revised Edition" }))]
    #[case("Tekkonkinkreet - Black & White 30th Anniversary Edition (2023) (Digital) (1r0n)", Some(Match { parsed: "Black & White 30th Anniversary Edition", raw: "Black & White 30th Anniversary Edition" }))]
    #[case("Tekkonkinkreet - (Black & White 30th Anniversary Edition) (2023) (Digital) (1r0n)", Some(Match { parsed: "Black & White 30th Anniversary Edition", raw: "(Black & White 30th Anniversary Edition)" }))]
    fn test_extract_edition(#[case] input: &str, #[case] expected: Option<Match<&str>>) {
        assert_eq!(extract_edition(input), expected);
    }

    #[rstest]
    #[case("Lover Boy v01 (2025) (Digital) (1r0n)", Some(Match { parsed: "1r0n", raw: "(1r0n)" }))]
    #[case("Witch and Mercenary v02 [Audiobook] [Seven Seas Siren] [Stick]", Some(Match { parsed: "Stick", raw: "[Stick]" }))]
    fn test_extract_group(#[case] input: &str, #[case] expected: Option<Match<&str>>) {
        assert_eq!(extract_group(input), expected);
    }

    #[rstest]
    #[case("Tekkonkinkreet - (Black & White 30th Anniversary Edition) (2023) (Digital) (1r0n)", "Tekkonkinkreet".to_string())]
    #[case("Youjo Senki | The Saga of Tanya the Evil Vol.26", "Youjo Senki".to_string())]
    fn test_cleanup(#[case] input: &str, #[case] expected: String) {
        assert_eq!(cleanup(input), expected);
    }
}
