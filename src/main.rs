use std::process::exit;
use std::env::args;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use regex::Regex;
use colored::*;

#[derive(Clone)]
struct Team {
    name: String,
    wins: i32,
    losses: i32,
    pfor: i32,
    pagainst: i32,
    pseed: i32, // Base case without previous seeding is 0
    ovr: i32,
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 3 || args.len() > 4 {
        eprintln!("usage: {} <team names> <game history> <optional: previous seeding>", args[0]);
        exit(1);
    }

    // Init teams
    let mut teams: Vec<(String, Team)> = get_team_data(&args[2], &mut populate_teams(&args[1]));

    // Seeding
    if args.len() == 4 {
        // In this case we print seeding based on previous seeding
        teams = get_prev_seeding(&args[3], &mut teams)
    }
    teams.sort_by(|(_, t1), (_, t2)| t1.ovr.partial_cmp(&t2.ovr).unwrap());
    teams.reverse();
    result(&mut teams);
}

// Populates a vector with tuples of (mascot, team data)
fn populate_teams(fp: &String) -> Vec<(String, Team)> {
    let rteam = Regex::new(r"(?<teamname>[A-z]+) (?<mascotname>[A-z]+)").unwrap();
    let mut teams: Vec<(String, Team)> = Vec::new();

    let file = File::open(&fp).expect(&format!("Couldn't open team name file {}", fp));
    let reader = BufReader::new(file);
    let mut linum: i32 = 1;
    for line in reader.lines() {
        let txt = line.expect(&format!("Couldn't read line {} in {}", linum, &fp));
        if let Some(caps) = rteam.captures(&txt) {
            if !has_mascot(&teams, &caps["mascotname"]) {
                let newteam = Team {
                    name: String::from(&caps["teamname"]),
                    wins: 0,
                    losses: 0,
                    pfor: 0,
                    pagainst: 0,
                    pseed: 0,
                    ovr: 0,
                };
                teams.push((caps["mascotname"].to_string(), newteam));
            } else {
                // Enforce unique mascot names
                eprintln!("Error: Team with mascot {} exists twice in {}", &caps["mascotname"], &fp);
                exit(1);
            }
        }
        linum += 1;
    }
    teams
}

// Requires a nonempty vector of (mascot, team data) tuples. Errors if a team
// with a nonexistent mascot is present.
fn get_team_data(fp: &str, teams: &mut Vec<(String, Team)>) -> Vec<(String, Team)> {
    let rgame = Regex::new(r"(?<gamenum>\d+)\. (?<wmascot>[A-z]+) def\. (?<lmascot>[A-z]+) (?<wscore>\d+)-(?<lscore>\d+)").unwrap();
    let file = File::open(&fp).expect(&format!("Couldn't open game history file {}", fp));
    let reader = BufReader::new(file);
    let mut linum: i32 = 1;
    for line in reader.lines() {
        let txt = line.expect(&format!("Couldn't read line {} in {}", linum, &fp));
        if let Some(caps) = rgame.captures(&txt) {
            // TODO: Make this more efficient
            // Winning team
            if let Some((_, wteam)) = teams.iter_mut().find(|(mascot, _)| mascot == &caps["wmascot"]) {
                wteam.wins += 1;
                // "For" points
                match &caps["wscore"].parse::<i32>() {
                    Err(why) => {
                        eprintln!("Couldn't parse the score {} for team {} in {}", &caps["wscore"], &caps["wmascot"], fp);
                        exit(1);
                    }
                    Ok(score) => {
                        wteam.pfor += score;
                    }
                }
                // "Against" points
                match &caps["lscore"].parse::<i32>() {
                    Err(why) => {
                        eprintln!("Couldn't parse the score {} for team {} in {}", &caps["lscore"], &caps["lmascot"], fp);
                        exit(1);
                    }
                    Ok(score) => {
                        wteam.pagainst += score;
                    }
                }

            } else {
                eprintln!("Team {} does not exist in {}", &caps["wmascot"], fp);
                exit(1);
            }
            // Losing team
            if let Some((_, lteam)) = teams.iter_mut().find(|(mascot, _)| mascot == &caps["lmascot"]) {
                lteam.losses += 1;
                // "For" points
                match &caps["wscore"].parse::<i32>() {
                    Err(why) => {
                        eprintln!("Couldn't parse the score {} for team {} in {}", &caps["wscore"], &caps["wmascot"], fp);
                        exit(1);
                    }
                    Ok(score) => {
                        lteam.pagainst += score;
                    }
                }
                // "Against" points
                match &caps["lscore"].parse::<i32>() {
                    Err(why) => {
                        eprintln!("Couldn't parse the score {} for team {} in {}", &caps["lscore"], &caps["lmascot"], fp);
                        exit(1);
                    }
                    Ok(score) => {
                        lteam.pfor += score;
                    }
                }
            }
        }
        linum += 1;
    }
    // Recompute ovr for all teams
    // TODO: Make this the "classic" seeding method and add team rank later
    // TODO: There has to be a better way to type this conversion. Maybe restructure the Team struct?
    for (mascot, team) in teams.iter_mut() {
        team.ovr = ((((team.wins as f32) / ((team.wins as f32) + (team.losses as f32))) + ((team.pfor as f32) / (team.pagainst as f32))) * 100.0) as i32;
    }
    teams.to_vec()
}

// Assign prev seeding to teams, if it exists
fn get_prev_seeding (fp: &str, teams: &mut Vec<(String, Team)>) -> Vec<(String, Team)> {
    let rseed = Regex::new(r"(?<seed>\d+)\.\s+(?<teamname>[A-z]+)\s+(?<mascotname>[A-z]+)").unwrap();
    let file = File::open(&fp).expect(&format!("Couldn't open previous seeding file {}", fp));
    let reader = BufReader::new(file);
    let mut linum = 1;
    for line in reader.lines() {
        let txt = line.expect(&format!("Couldn't read line {} in {}", linum, fp));
        if let Some(caps) = rseed.captures(&txt) {
            if let Some((_, team)) = teams.iter_mut().find(|(mascot, _)| mascot == &caps["mascotname"]) {
                match &caps["seed"].parse::<i32>() {
                    Err(why) => {
                        eprintln!("Couldn't parse the seed {} for team {} in {}", &caps["seed"], &caps["mascotname"], fp);
                        exit(1);
                    }
                    Ok(seed) => team.pseed = *seed,
                }
            } else {
                eprintln!("Team {} does not exist in {}", &caps["mascotname"], fp);
                exit(1);
            }
        }
        linum += 1;
    }
    teams.to_vec()
}

// Prints seeding, data, and deltas (if applicable)
// Assumes the vector passed in is sorted by OVR // TODO: Implement for non-Classic seeding
// TODO: Abs val the delta; finish the full string; add colors
fn result(teams: &mut Vec<(String, Team)>) -> () {
    let mut i: i32 = 1;
    for (mascot, team) in teams.iter() {
        let mut sdelta: ColoredString;
        let sseed = format!("{}.", i);
        let wl = format!("{}-{}", team.wins, team.losses);
        let pfa = format!("{}-{}", team.pfor, team.pagainst);
        // TODO same redundant fix as needed above. change typing in the Team struct
        let wpct = format!("% {:.3}", (team.wins as f32) / ((team.wins as f32) + (team.losses as f32)));
        let far = format!("F/A% {:.3}", (team.pfor as f32) / (team.pagainst as f32));
        let ovr = format!("OVR: {:.1}", team.ovr);
        let ppg = format!("{:.1}", (team.pfor as f32) / ((team.wins as f32) + (team.losses as f32)));
        let d = i - team.pseed;
        if team.pseed == 0 || d == 0 {
            sdelta = "(-)".normal();
        } else if d < 0 {
            sdelta = format!("(▼ {})", d).red();
        } else {
            sdelta = format!("(▲ {})", d).green();
        }
        // TODO: Replace hardcoded widths with variables based on max length of possible args
        println!("{:<4} {:<15} {:<15} {:<7} {:<5} {:<9} {:<8} {:<8} {:<8}", sseed.magenta(), team.name, mascot, sdelta, wl.blue(), pfa.blue(), wpct.cyan(), far.cyan(), ovr.green());
        i += 1;
    }
}

fn has_mascot(teams: &Vec<(String, Team)>, target_mascot: &str) -> bool {
    teams.iter().any(|(mascot, _)| mascot == target_mascot)
}