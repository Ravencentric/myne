mod parsers;
mod publishers;
mod repr;

use std::env;

use clap::Parser;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use parsers::Match;
use repr::PyRepr;

#[pyclass(module = "myne", frozen, eq, hash, get_all)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct Book {
    title: String,
    digital: bool,
    edited: bool,
    compilation: bool,
    pre: bool,
    revision: u8,
    volume: Option<String>,
    chapter: Option<String>,
    group: Option<String>,
    year: Option<u16>,
    edition: Option<String>,
    extension: Option<String>,
    publisher: Option<String>,
}

#[pymethods]
impl Book {
    #[new]
    #[pyo3(signature = (*, title, digital=false, edited=false, compilation=false, pre=false, revision=1, volume=None, chapter=None, group=None, year=None, edition=None, extension=None, publisher=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        title: String,
        digital: bool,
        edited: bool,
        compilation: bool,
        pre: bool,
        revision: u8,
        volume: Option<String>,
        chapter: Option<String>,
        group: Option<String>,
        year: Option<u16>,
        edition: Option<String>,
        extension: Option<String>,
        publisher: Option<String>,
    ) -> Self {
        Book {
            title,
            digital,
            edited,
            compilation,
            pre,
            revision,
            volume,
            chapter,
            group,
            year,
            edition,
            extension,
            publisher,
        }
    }

    #[staticmethod]
    #[pyo3(signature = (filename, /))]
    fn parse(filename: &str) -> Book {
        // Initialize a mutable string with the filename. As metadata fields are
        // successfully extracted in subsequent steps, their raw matched strings
        // will be removed from this `leftover` string, simplifying it
        // and eventually leaving behind the title.
        let mut leftover = filename.trim().replace('_', " ");

        // First, separate the file extension from the main part of the name (the stem).
        let mut extension: Option<String> = None;
        if let Some(Match { parsed, raw }) = parsers::extract_extension(&leftover) {
            extension = Some(parsed.to_string());
            leftover = leftover.replace(raw, "");
        };

        // Attempt to extract the publisher information from the stem.
        let mut publisher: Option<String> = None;
        if let Some(Match { parsed, raw }) = parsers::extract_publisher(&leftover) {
            publisher = Some(parsed.to_string());
            leftover = leftover.replace(raw, "");
        };

        // Determine the digital and compilation status. This uses a prioritized
        // check: first look for a combined "Digital Compilation" marker, then
        // a simple "Digital" marker.
        let digital: bool;
        let mut compilation: bool = false;
        if let Some(Match { parsed: _, raw }) = parsers::extract_digital_compilation(&leftover) {
            digital = true;
            compilation = true;
            leftover = leftover.replace(raw, "");
        } else if let Some(Match { parsed: _, raw }) = parsers::extract_digital(&leftover) {
            digital = true;
            compilation = false;
            leftover = leftover.replace(raw, "");
        } else {
            digital = false
        };

        // Check for an "Edited" marker and set the edited flag.
        let mut edited: bool = false;
        if let Some(Match { parsed: _, raw }) = parsers::extract_edited(&leftover) {
            edited = true;
            leftover = leftover.replace(raw, "");
        };

        // Check for a "PRE" marker and set the pre flag.
        let mut pre: bool = false;
        if let Some(Match { parsed: _, raw }) = parsers::extract_pre(&leftover) {
            pre = true;
            leftover = leftover.replace(raw, "");
        };

        // Extract the revision number, defaulting to 1.
        let mut revision: u8 = 1;
        if let Some(Match { parsed, raw }) = parsers::extract_revision(&leftover) {
            revision = parsed;
            leftover = leftover.replace(raw, "");
        };

        // Extract the year from the stem.
        let mut year: Option<u16> = None;
        if let Some(Match { parsed, raw }) = parsers::extract_year(&leftover) {
            year = Some(parsed);
            leftover = leftover.replace(raw, "");
        };

        // Extract the edition information (e.g., "Deluxe Edition").
        let mut edition: Option<String> = None;
        if let Some(Match { parsed, raw }) = parsers::extract_edition(&leftover) {
            edition = Some(parsed.to_string());
            leftover = leftover.replace(raw, "");
        };

        // Extract the group information.
        let mut group: Option<String> = None;
        if let Some(Match { parsed, raw }) = parsers::extract_group(&leftover) {
            group = Some(parsed.to_string());
            leftover = leftover.replace(raw, "");
        };

        // Extract the volume number/string.
        let mut volume: Option<String> = None;
        if let Some(Match { parsed, raw }) = parsers::extract_volume(&leftover) {
            volume = Some(parsed.to_string());
            leftover = leftover.replace(raw, "");
        };

        // Extract the chapter number/string.
        let mut chapter: Option<String> = None;
        if let Some(Match { parsed, raw }) = parsers::extract_chapter(&leftover) {
            chapter = Some(parsed.to_string());
            leftover = leftover.replace(raw, "");
        };

        // After removing all identified metadata fields from `leftover`, the
        // remaining string represents the title.
        let title = parsers::cleanup(&leftover);

        Book {
            title,
            digital,
            edited,
            compilation,
            pre,
            revision,
            volume,
            chapter,
            group,
            year,
            edition,
            extension,
            publisher,
        }
    }

    #[staticmethod]
    #[pyo3(signature = (data, /))]
    fn from_json(data: &str) -> PyResult<Book> {
        let book = serde_json::from_str(data).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(book)
    }

    fn to_json(&self) -> String {
        // UNWRAP SAFETY: unwrap is infallible here,
        // because &self derives (De)Serialize and it's
        // impossible for a bad Book to ever be constructed
        // in the first place.
        serde_json::to_string_pretty(&self).unwrap()
    }

    fn __repr__(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("title={}", self.title.__repr__()));

        if self.digital {
            parts.push(format!("digital={}", self.digital.__repr__()));
        }
        if self.edited {
            parts.push(format!("edited={}", self.edited.__repr__()));
        }
        if self.compilation {
            parts.push(format!("compilation={}", self.compilation.__repr__()));
        }
        if self.pre {
            parts.push(format!("pre={}", self.pre.__repr__()));
        }
        if self.revision != 1 {
            parts.push(format!("revision={}", self.revision.__repr__()));
        }
        if let Some(ref volume) = self.volume {
            parts.push(format!("volume={}", volume.__repr__()));
        }
        if let Some(ref chapter) = self.chapter {
            parts.push(format!("chapter={}", chapter.__repr__()));
        }
        if let Some(ref group) = self.group {
            parts.push(format!("group={}", group.__repr__()));
        }
        if let Some(ref year) = self.year {
            parts.push(format!("year={}", year.__repr__()));
        }
        if let Some(ref edition) = self.edition {
            parts.push(format!("edition={}", edition.__repr__()));
        }
        if let Some(ref extension) = self.extension {
            parts.push(format!("extension={}", extension.__repr__()));
        }
        if let Some(ref publisher) = self.publisher {
            parts.push(format!("publisher={}", publisher.__repr__()));
        }

        format!("Book({})", parts.join(", "))
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

// Stolen from https://github.com/astral-sh/uv/blob/96cfca1c8fe711d24215f9d1bb91cea7002aa087/crates/uv-cli/src/lib.rs#L69-L74
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Parser, Debug)]
#[command(version, long_about = None, styles = STYLES)]
/// Parser for manga and light novel filenames.
struct MyneCli {
    /// The manga or light novel filename to parse.
    filename: String,
}

#[pyfunction]
fn script() {
    // We need to skip the first entry in std::env::args_os,
    // because it ends up being the path to this script itself.
    // Example:
    // ```console
    // ❯ myne
    // MyneCli { name: "C:\\Users\\raven\\dev\\github\\myne\\.venv\\Scripts\\myne.exe" }
    // ```
    let args_os = env::args_os().skip(1);
    let args = MyneCli::parse_from(args_os);
    let book = Book::parse(&args.filename);
    println!("{}", book.to_json());
}

/// Parser for manga and light novel filenames.
#[pymodule(gil_used = false)]
fn myne(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Book>()?;
    m.add_function(wrap_pyfunction!(script, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__all__", ("Book", "__version__"))?;
    Ok(())
}
