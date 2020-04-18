use crate::errors::{FormatError, SourceError};
use crate::input::{Formatter, SortOrder, Source};
use std::fs;
use std::fs::ReadDir;
use regex::Regex;
use std::error::Error;
use std::iter::IntoIterator;
use std::path::Path;
use std::vec::IntoIter;

pub enum InputStream {
    VectorStream(IntoIter<(String, String)>),
    DirectoryStream(Formatter, Regex, bool, ReadDir),
}

impl InputStream {
    pub fn try_from(
        source: Source,
        formatter: Option<Formatter>,
        preserve_extension: bool,
    ) -> Result<Self, Box<dyn Error>> {
        if let Source::Map(map) = source {
            return Ok(Self::VectorStream(map.into_iter()));
        }

        let formatter = formatter.ok_or(FormatError::EmptyFormatter)?;

        let mut entries = fs::read_dir(".")?;

        if let Source::Sort(order) = source {
            let mut map = Vec::new();
            let mut inputs = Vec::new();
            while let Some(entry) = entries.next() {
                if let Ok(entry) = entry {
                    inputs.push(entry.file_name().to_string_lossy().to_string());
                }
            }
            inputs.sort_by(|a, b| {
                if order == SortOrder::Asc {
                    a.to_lowercase().as_str().cmp(b.to_lowercase().as_str())
                } else {
                    b.to_lowercase().as_str().cmp(a.to_lowercase().as_str())
                }
            });
            for (i, input) in inputs.into_iter().enumerate() {
                let index = (i + 1).to_string();
                let mut output = formatter.format(vec![input.as_str(), index.as_str()].as_slice());
                if preserve_extension {
                    if let Some(extension) = Path::new(input.as_str()).extension() {
                        output.push('.');
                        output.push_str(extension.to_str().unwrap_or_default());
                    }
                }
                map.push((input, output));
            }
            return Ok(Self::VectorStream(map.into_iter()));
        }

        if let Source::Regex(re) = source {
            return Ok(Self::DirectoryStream(
                formatter,
                re,
                preserve_extension,
                entries,
            ));
        }

        Err(Box::new(SourceError::new(String::from("unknown source"))))
    }
}

impl Iterator for InputStream {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::VectorStream(ref mut iter) => return iter.next(),
            Self::DirectoryStream(ref formatter, ref re, preserve_extension, iter) => {
                loop {
                    match iter.next() {
                        Some(Ok(entry)) => {
                            let input = entry.file_name().to_string_lossy().to_string();
                            if let Some(cap) = re.captures(input.as_str()) {
                                let vars: Vec<&str> = cap
                                    .iter()
                                    .map(|c| c.map(|c| c.as_str()).unwrap_or_default())
                                    .collect();
                                let mut output = formatter.format(vars.as_slice());
                                if *preserve_extension {
                                    if let Some(extension) = Path::new(input.as_str()).extension() {
                                        output.push('.');
                                        output.push_str(extension.to_str().unwrap_or_default());
                                    }
                                }
                                return Some((input, output));
                            }
                        }
                        Some(Err(_)) => { continue; }
                        None => { return None; }
                    }
                }
            }
        }
    }
}
