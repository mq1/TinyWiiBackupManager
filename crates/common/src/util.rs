// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Display;

fn is_valid_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || " !#$%&'()+,-.;=@^_`{}~".contains(c)
}

pub fn sanitize_title(ascii_title: &str) -> String {
    let mut title = ascii_title
        .chars()
        .skip_while(|c| !c.is_ascii_alphanumeric())
        .filter(|&c| is_valid_char(c))
        .take(64)
        .collect::<String>();

    // Remove trailing whitespace in-place
    title.truncate(title.trim_end().len());

    if title.is_empty() {
        title.push_str("game");
    }

    title
}

pub fn make_game_dir_name<GameID: AsRef<str> + Display + Copy>(
    game_id: GameID,
    fallback_title: &str,
) -> String {
    let game_title = {
        let ascii_title = twbm_idmap::get_ascii_title(game_id).unwrap_or(fallback_title);
        sanitize_title(ascii_title)
    };

    format!("{game_title} [{game_id}]")
}

pub fn make_search_term(patterns: &[&str]) -> Box<str> {
    let len_hint =
        patterns.iter().map(|p| p.len()).sum::<usize>() + patterns.len().saturating_sub(1);

    let mut term = String::with_capacity(len_hint);

    for (i, p) in patterns.iter().enumerate() {
        if i > 0 {
            term.push('\0');
        }

        term.push_str(p);
    }

    if term.is_ascii() {
        term.make_ascii_lowercase();
    } else {
        term = term.to_lowercase();
    };

    term.into()
}
