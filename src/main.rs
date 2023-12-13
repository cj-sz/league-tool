use std::process::exit;
use std::env::args;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use regex::Regex;
use colored::*;

#[derive(Clone)]
#[derive(PartialEq)]
struct Team {
    name: String,
    wins: f32,
    losses: f32,
    ties: f32,
    pfor: f32,
    pagainst: f32,
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
                    wins: 0.0,
                    losses: 0.0,
                    ties: 0.0,
                    pfor: 0.0,
                    pagainst: 0.0,
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
// TODO: Hard enforce that if scores =, must say "tie", and if !=, must say "def."
fn get_team_data(fp: &str, teams: &mut Vec<(String, Team)>) -> Vec<(String, Team)> {
    let rgame = Regex::new(r"(?<gamenum>\d+)\. (?<leftmascot>[A-z]+) (def\.|tie) (?<rightmascot>[A-z]+) (?<leftscore>\d+)-(?<rightscore>\d+)").unwrap();
    let file = File::open(&fp).expect(&format!("Couldn't open game history file {}", fp));
    let reader = BufReader::new(file);
    let mut linum = 1;
    for line in reader.lines() {
        let txt = line.expect(&format!("Couldn't read line {} in {}", linum, &fp));
        if let Some(caps) = rgame.captures(&txt) {
            let leftscore_result = caps["leftscore"].parse::<f32>();
            let rightscore_result = caps["rightscore"].parse::<f32>();
            if leftscore_result.is_err() || rightscore_result.is_err() {
                eprintln!("Couldn't parse scores on line {} of {}", linum, fp);
                exit(1);
            }
            let leftscore = leftscore_result.unwrap();
            let rightscore = rightscore_result.unwrap();
            if let Some((_, leftteam)) = teams.iter_mut().find(|(mascot, _)| mascot == &caps["leftmascot"]) {
                if leftscore == rightscore {
                    leftteam.ties += 1.0;
                } else {
                    leftteam.wins += 1.0;
                }
                leftteam.pfor += leftscore;
                leftteam.pagainst += rightscore;
            } else {
                eprintln!("Team {} does not exist on line {} of {}", &caps["leftmascot"], linum, fp);
                exit(1);
            }
            if let Some((_, rightteam)) = teams.iter_mut().find(|(mascot, _)| mascot == &caps["rightmascot"]) {
                if leftscore == rightscore {
                    rightteam.ties += 1.0;
                } else {
                    rightteam.losses += 1.0;
                }
                rightteam.pfor += rightscore;
                rightteam.pagainst += leftscore;
            } else {
                eprintln!("Team {} does not exist on line {} of {}", &caps["rightmascot"], linum, fp);
                exit(1);
            }
        }
        linum += 1;
    }
    // Recompute ovr for all teams
    // TODO: Make this the "classic" seeding method and add team rank later
    for (_, team) in teams.iter_mut() {
        team.ovr = (((team.wins / (team.wins + team.losses + team.ties)) + (team.pfor / team.pagainst)) * 100.0) as i32;
    }
    teams.to_vec()
}

// Assign prev seeding to teams, if it exists. Otherwise remains 0.
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
                        eprintln!("Couldn't parse the seed {} for team {} in {}: {}", &caps["seed"], &caps["mascotname"], fp, why);
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
fn result(teams: &mut Vec<(String, Team)>) -> () {
    let mut i: i32 = 1;
    for (mascot, team) in teams.iter() {
        let sdelta: ColoredString;
        let sseed = format!("{}.", i);
        let wlt = format!("{}-{}-{}", team.wins, team.losses, team.ties);
        let pfa = format!("{}-{}", team.pfor, team.pagainst);
        let wpct = format!("% {:.3}", team.wins / (team.wins + team.losses + team.ties));
        let far = format!("F/A% {:.3}", team.pfor / team.pagainst);
        let ovr = format!("OVR: {:.1}", team.ovr);
        let ppg = format!("PPG: {:.1}", team.pfor / (team.wins + team.losses + team.ties));
        let d = i - team.pseed;
        if team.pseed == 0 || d == 0 {
            sdelta = "(-)".normal();
        } else if d < 0 {
            sdelta = format!("(▲ {})", -1 * d).green();
        } else {
            sdelta = format!("(▼ {})", d).red();
        }
        // TODO: Replace hardcoded widths with variables based on max length of possible args
        println!("{:<4} {:<20} {:<20} {:<12} {:<16} {:<12} {:<12} {:<12} {:<12} {:<12}", sseed.magenta(), team.name, mascot, sdelta, wlt.blue(), pfa.blue(), wpct.cyan(), far.cyan(), ppg.cyan(), ovr.green());
        i += 1;
    }
}

fn has_mascot(teams: &Vec<(String, Team)>, target_mascot: &str) -> bool {
    teams.iter().any(|(mascot, _)| mascot == target_mascot)
}