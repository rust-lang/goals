use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use spanned::{Result, Spanned};

use crate::{
    config::Configuration,
    goal::{Status, TeamAsk},
    team::TeamName,
    util,
};

/// Format a set of team asks into a table, with asks separated by team and grouped by kind.
///
/// output looks like
///
/// ```ignore
/// | Goal      | [DMS](#discussion-and-moral-support) | [SR](#standard reviews) |
/// | :---      | :--                                  | :--                     |
/// | Foo       | ✅                                   | ✅ (notes)               |
/// | Bar (Baz) | ✅                                   | ✅ (\*1)                 |
///
/// \*1: ... longer notes that would not fit ...
/// ```
pub fn format_team_asks(asks_of_any_team: &[&TeamAsk]) -> Result<String> {
    use std::fmt::Write;

    const CHECK: &str = "✅";

    let mut output = String::new();

    let all_teams: BTreeSet<&TeamName> = asks_of_any_team
        .iter()
        .flat_map(|a| &a.teams)
        .copied()
        .collect();

    // The set of configured team asks
    let config = Configuration::get();

    for team_name in all_teams {
        let asks_of_this_team: Vec<_> = asks_of_any_team
            .iter()
            .filter(|a| a.teams.contains(&team_name))
            .collect();

        let team_data = team_name.data();
        write!(output, "\n#### {} team\n", team_data.name)?;

        // We will accumulate footnotes when we encounter comments that are too long.
        let mut footnotes = util::Footnotes::new();

        // These are things like "discussion and moral support". They are extracted from
        // the configuration. We prune out the ones that do not appear in the asks for a particular team.
        let ask_headings = config
            .team_asks
            .iter()
            .filter(|&(ask_kind, ask_details)| {
                !ask_details.elide
                    && asks_of_this_team
                        .iter()
                        .any(|a| &a.ask_description == ask_kind)
            })
            .map(|(ask_kind, _)| ask_kind)
            .collect::<Vec<_>>();
        let empty_row = || {
            (0..ask_headings.len())
                .map(|_| "".to_string())
                .collect::<Vec<_>>()
        };

        // Collect the asks by goal. The `rows` map goes from goal title to a row with entries
        let mut goal_rows: BTreeMap<GoalData<'_>, Vec<String>> = BTreeMap::default();
        for ask in &asks_of_this_team {
            let goal_data = GoalData::new(ask)?;

            let row = goal_rows.entry(goal_data).or_insert_with(empty_row);

            let Some(index) = ask_headings.iter().position(|&h| h == &ask.ask_description) else {
                // Some asks are not included in the table
                assert!(
                    config.team_asks[&ask.ask_description].elide,
                    "ask {} has no index but is not elided",
                    ask.ask_description
                );
                continue;
            };

            let text = if !ask.notes.is_empty() {
                &ask.notes
            } else {
                CHECK
            };

            let cell = footnotes.maybe_footnote(text, &ask.link_path);

            if !row[index].is_empty() {
                row[index] = format!("{} {}", row[index], cell);
            } else {
                row[index] = cell;
            }
        }

        // Ensure that we have an entry for the "meta-goal", even if there are no asks.
        for ask in &asks_of_this_team {
            let mut goal_data = GoalData::new(ask)?;
            goal_data.subgoal_title = None;
            goal_rows.entry(goal_data).or_insert_with(empty_row);
        }

        // Create the table itself.
        let table = {
            let headings = std::iter::once(Spanned::here("Goal".to_string()))
                .chain(ask_headings.iter().map(|&ask_kind| {
                    Spanned::here(format!(
                        "[{team_ask_short}][valid_team_asks]", // HACK: This should not be hardcoded in the code.
                        team_ask_short = config.team_asks[ask_kind].short,
                    ))
                })) // e.g. "discussion and moral support"
                .collect::<Vec<Spanned<String>>>();

            let rows = goal_rows.into_iter().map(|(goal_data, goal_columns)| {
                std::iter::once(goal_data.goal_title())
                    .chain(goal_columns)
                    .map(Spanned::here)
                    .collect::<Vec<Spanned<String>>>()
            });

            std::iter::once(headings).chain(rows).collect::<Vec<_>>()
        };

        write!(output, "{}", util::format_table(&table))?;

        footnotes.write_to(&mut output)?;
    }

    Ok(output)
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Copy, Clone, Debug)]
struct GoalData<'g> {
    goal_title: &'g String,
    subgoal_title: Option<&'g String>,
    link: &'g PathBuf,
    status: Status,
}

impl<'g> GoalData<'g> {
    fn new(ask: &'g TeamAsk) -> Result<Self> {
        match &ask.goal_titles[..] {
            [goal_title] => Ok(Self {
                goal_title,
                subgoal_title: None,
                link: &ask.link_path,
                status: ask.status,
            }),
            [goal_title, subgoal_title] => Ok(Self {
                goal_title,
                subgoal_title: Some(subgoal_title),
                link: &ask.link_path,
                status: ask.status,
            }),
            _ => spanned::bail!(
                ask.goal_titles[3],
                "expected either 1 or 2 goal titles, not {}",
                ask.goal_titles.len(),
            ),
        }
    }

    fn goal_title(&self) -> String {
        let title = self.subgoal_title.unwrap_or(self.goal_title);
        let title = self.status.decorate_title(title);
        util::goal_title_cell(
            &title,
            self.link,
            self.subgoal_title.map(|_| title.as_str()),
        )
    }
}
