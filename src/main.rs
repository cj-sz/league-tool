use std::process::exit;
use std::env::args;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use regex::Regex;

// In the readme also note that we eventually want to convert to the team ranking algorithm.

#[derive(Clone)]
struct Team {
    name: String,
    wins: i32,
    losses: i32,
    pfor: i32,
    pagainst: i32,
}

fn main() {
    // Validate arg count
    let args: Vec<String> = args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("usage: {} <team names> <game history> <optional: previous seeding>", args[0]);
        exit(1);
    }

    // Required regex
    let rseed = Regex::new(r"(?<seed>\d+)\.\s+(?<teamname>[A-z]+)\s+(?<mascotname>[A-z]+)").unwrap();
    // TODO: Seeding regex with the stuff wanted

    // Init teams
    let mut teams: Vec<(String, Team)> = get_team_data(&args[2], &mut populate_teams(&args[1]));

    // Compute seeding for everything

    // Seeding stuff
    if args.len() == 3 {
        // In this case we print seeding based on previous seeding
        println!("Seeding support TODO.");
    } else {
        // In this case we just want to do the seeding from scratch, and not compare it.
    }
}

fn populate_teams(fp: &String) -> Vec<(String, Team)> {
    let rteam = Regex::new(r"(?<teamname>[A-z]+) (?<mascotname>[A-z]+)").unwrap();
    let mut teams: Vec<(String, Team)> = Vec::new();

    // Validity of file
    let file = match File::open(&fp) {
        Err(why) => {
            eprintln!("Couldn't open team data file {}: {}", fp, why);
            exit(1);
        }
        Ok(file) => file
    };

    // Populate the vector with default team data
    let reader = BufReader::new(file);
    let mut linum: i32 = 1;
    for line in reader.lines() {
        let txt = match line {
            Err(why) => {
                eprintln!("Couldn't read line {} in {}: {}", linum, &fp, why);
                exit(1);
            }
            Ok(txt) => txt
        };

        if let Some(caps) = rteam.captures(&txt) {
             // need to remake the below function
            if !has_mascot(&teams, &caps["mascotname"]) {
                let newteam = Team {
                    name: String::from(&caps["teamname"]),
                    wins: 0,
                    losses: 0,
                    pfor: 0,
                    pagainst: 0,
                };
                teams.push((caps["mascotname"].to_string(), newteam));
            } else {
                // Exit if a duplicate mascot exists
                eprintln!("Error: Team with mascot {} exists twice in {}", &caps["mascotname"], &fp);
                exit(1);
            }
            linum += 1;
        }
    }

    return teams
}

fn get_team_data(fp: &str, teams: &mut Vec<(String, Team)>) -> Vec<(String, Team)> {
    let rgame = Regex::new(r"(?<gamenum>\d+)\. (?<wmascot>[A-z]+) def\. (?<lmascot>[A-z]+) (?<wscore>\d+)-(?<lscore>\d+)").unwrap();

    let file = match File::open(&fp) {
        Err(why) => {
            eprintln!("Couldn't open game history file {}: {}", fp, why);
            exit(1);
        }
        Ok(file) => file
    };

    let reader = BufReader::new(file);
    let mut linum: i32 = 1;
    for line in reader.lines() {
        let txt = match line {
            Err(why) => {
                eprintln!("Couldn't read line {} in {}: {}", linum, &fp, why);
                exit(1);
            }
            Ok(txt) => txt
        };

        if let Some(caps) = rgame.captures(&txt) {
            // Eventually this entire section can be made more efficient
            // First find the winning team:
            if let Some((_, wteam)) = teams.iter_mut().find(|(mascot, _)| mascot == &caps["wmascot"]) {
                wteam.wins += 1;
                // Add the for points
                match &caps["wscore"].parse::<i32>() {
                    Err(why) => {
                        eprintln!("Couldn't parse the score {} for team {} in {}", &caps["wscore"], &caps["wmascot"], fp);
                        exit(1);
                    }
                    Ok(score) => {
                        wteam.pfor += score;
                    }
                }
                // Add the against points
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
                // For now, we simply state that the team does not exist
                eprintln!("Team {} does not exist in {}", &caps["wmascot"], fp);
                exit(1);
            }

            // Then do the same for the losing team
            if let Some((_, lteam)) = teams.iter_mut().find(|(mascot, _)| mascot == &caps["lmascot"]) {
                lteam.losses += 1;
                // Add the against points 
                match &caps["wscore"].parse::<i32>() {
                    Err(why) => {
                        eprintln!("Couldn't parse the score {} for team {} in {}", &caps["wscore"], &caps["wmascot"], fp);
                        exit(1);
                    }
                    Ok(score) => {
                        lteam.pagainst += score;
                    }
                }
                // Add the for points
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
    }

    return teams.to_vec()
}

fn has_mascot(teams: &Vec<(String, Team)>, target_mascot: &str) -> bool {
    teams.iter().any(|(mascot, _)| mascot == target_mascot)
}