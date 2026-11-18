// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

const USAGE: &str = "Usage: twbm-idmap <GAMEID>";

fn main() {
    let Some(game_id) = std::env::args().nth(1) else {
        eprintln!("{USAGE}");
        std::process::exit(1);
    };

    let title = twbm_idmap::get_title(&game_id);
    println!("Title: {title:?}");

    #[cfg(feature = "ascii-titles")]
    {
        let ascii_title = twbm_idmap::get_ascii_title(&game_id);
        println!("ASCII Title: {ascii_title:?}");
    }

    #[cfg(feature = "gamehacking")]
    {
        let ghid = twbm_idmap::get_ghid(&game_id);
        println!("GameHacking ID: {ghid:?}");
    }
}
