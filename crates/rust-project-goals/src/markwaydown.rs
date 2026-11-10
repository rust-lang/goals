//! Arguably the worst markdown parser ever. Extracts exactly the things we care about.

use std::{fmt::Display, path::Path};

use spanned::{Error, Result, Spanned};

use crate::title::Title;
use crate::util;

/// A "section" is a piece of markdown that begins with `##` and which extends until the next section.
/// Note that we don't track the hierarchical structure of sections in particular.
#[derive(Debug)]
pub struct Section {
    /// Number of hashes
    pub level: usize,

    /// Title of the section -- what came after the `#` in the markdown.
    pub title: Spanned<Title>,

    /// Markdown text until start of next section, excluding tables
    pub text: Spanned<String>,

    /// Tables are parsed and stored here
    pub tables: Vec<Spanned<Table>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Table {
    pub header: Vec<Spanned<Title>>,
    pub rows: Vec<Vec<Spanned<String>>>,
}

pub fn parse(path: impl AsRef<Path>) -> Result<Vec<Section>> {
    let path = path.as_ref();
    let text = Spanned::read_str_from_file(path).transpose()?;
    parse_text(text.as_ref().map(|s| s.as_ref()))
}

pub fn parse_text(text: Spanned<&str>) -> Result<Vec<Section>> {
    let mut result = vec![];
    let mut open_section = None;
    let mut open_table = None;

    for line in text.lines() {
        let line = line.to_str().unwrap();
        let categorized = categorize_line(line.clone());
        // eprintln!("line = {:?}", line.content);
        // eprintln!("categorized = {:?}", categorized);
        match categorized {
            CategorizeLine::Title(level, title) => {
                close_section(&mut result, &mut open_section, &mut open_table);
                open_section = Some(Section {
                    level,
                    title,
                    text: Default::default(),
                    tables: vec![],
                });
            }
            CategorizeLine::TableRow(mut row) => {
                if open_section.is_none() {
                    // create an "anonymous" section to house the table
                    open_section = Some(Section {
                        level: 0,
                        title: Default::default(),
                        text: Default::default(),
                        tables: vec![],
                    });
                }

                if let Some(table) = &mut open_table {
                    if row.len() > table.header.len() {
                        spanned::bail!(
                            row[table.header.len()],
                            "too many columns in table, expected no more than {}",
                            table.header.len()
                        );
                    }

                    while row.len() < table.header.len() {
                        row.push(Spanned::here(String::new()));
                    }

                    table.span.bytes.end = line.span.bytes.end;
                    table.content.rows.push(row);
                } else {
                    open_table = Some(Spanned::new(
                        Table {
                            header: row.into_iter().map(|s| s.map(Title::new)).collect(),
                            rows: vec![],
                        },
                        line.span.clone(),
                    ));
                }
            }
            CategorizeLine::TableDashRow(dashes) => {
                if let Some(table) = &mut open_table {
                    if table.header.len() != dashes.len() {
                        spanned::bail!(
                            dashes.last().unwrap(),
                            "invalid number of columns in table, expected {}",
                            table.header.len()
                        );
                    }

                    if let Some(first) = table.rows.first() {
                        return Err(Error::new_str(
                            dashes[0]
                                .as_ref()
                                .map(|_| "did not expect table header here"),
                        )
                        .wrap_str(first[0].as_ref().map(|_| "already saw table row here")));
                    }

                    table.span.bytes.end = line.span.bytes.end;
                } else {
                    spanned::bail!(dashes[0], "did not expect table header here",);
                }
            }
            CategorizeLine::Other => {
                close_table(&mut open_section, &mut open_table);
                if let Some(section) = open_section.as_mut() {
                    section.text.span.bytes.end = line.span.bytes.end;
                    section.text.content.push_str(&line);
                    section.text.content.push('\n');
                }
            }
        }
    }

    close_section(&mut result, &mut open_section, &mut open_table);

    Ok(result)
}

fn close_table(open_section: &mut Option<Section>, open_table: &mut Option<Spanned<Table>>) {
    if let Some(table) = open_table.take() {
        open_section.as_mut().unwrap().tables.push(table);
    }
}

fn close_section(
    result: &mut Vec<Section>,
    open_section: &mut Option<Section>,
    open_table: &mut Option<Spanned<Table>>,
) {
    close_table(open_section, open_table);
    if let Some(section) = open_section.take() {
        result.push(section);
    }
}

#[derive(Debug)]
enum CategorizeLine {
    Title(usize, Spanned<Title>),
    TableRow(Vec<Spanned<String>>),
    TableDashRow(Vec<Spanned<()>>),
    Other,
}

fn categorize_line(line: Spanned<&str>) -> CategorizeLine {
    // Trimming _may_ slightly mess up spans sometimes, but it's preferable to e.g. failing to parse
    // tables due to unexpected spaces.
    if line.starts_with("#") {
        let level = line.chars().take_while(|ch| **ch == '#').count();
        CategorizeLine::Title(
            level,
            line.trim_start_matches('#')
                .trim()
                .to_string()
                .map(Title::new),
        )
    } else if let Some(line) = line
        .trim()
        .strip_prefix("|")
        .and_then(|line| line.strip_suffix("|"))
    {
        let columns = line.split('|').map(|s| s.trim());
        if columns
            .clone()
            .all(|s| s.chars().all(|c| *c == '-' || *c == ':'))
        {
            CategorizeLine::TableDashRow(columns.map(|s| s.map(drop)).collect())
        } else {
            CategorizeLine::TableRow(columns.map(|s| s.to_string()).collect())
        }
    } else {
        CategorizeLine::Other
    }
}

impl Table {
    /// For a "key-value" table (like metadata), find an existing row
    /// where the first column (the "key") is `row_key` and modify its second column (the "value")
    /// to be `row_value`. If no row exists with key `row_key`, then add a new row.
    pub fn add_key_value_row(&mut self, row_key: &str, row_value: &impl Display) {
        assert_eq!(self.header.len(), 2);

        match self.rows.iter_mut().find(|row| row[0] == row_key) {
            Some(row) => {
                // FIXME(oli-obk): get proper spans
                row[1] = Spanned::here(row_value.to_string());
            }

            None => {
                self.rows.push(vec![
                    Spanned::here(row_key.to_string()),
                    Spanned::here(row_value.to_string()),
                ]);
            }
        }
    }

    /// Modify `path` to replace the lines containing this table with `new_table`.
    pub fn overwrite_in_path(this: &Spanned<Self>, path: &Path, new_table: &Table) -> Result<()> {
        let full_text = std::fs::read_to_string(path)?;
        let table_span = &this.span.bytes;

        let table_text = &full_text[table_span.start..table_span.end];
        assert!(
            table_text.starts_with("|") && table_text.ends_with("|"),
            "table_text doesn't appear to be a table: {table_text:?}"
        );

        let mut new_text = full_text[..table_span.start].to_string();

        let table_text = {
            let mut new_rows = vec![new_table
                .header
                .iter()
                .cloned()
                .map(|s| s.map(|t| t.displayable().to_string()))
                .collect()];
            new_rows.extend(new_table.rows.iter().cloned());
            util::format_table(&new_rows)
        };
        new_text.push_str(&table_text);
        new_text.push_str(&full_text[table_span.end..]);

        std::fs::write(path, new_text)?;

        Ok(())
    }
}
