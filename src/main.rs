use serde;
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use clap::{Arg, App};

type Standings = HashMap<String, f32>;

#[derive(serde::Deserialize, std::marker::Copy, std::clone::Clone)]
enum SeriesKind {
    Bo1,
    Bo3,
    Bo5,
}

#[derive(serde::Deserialize)]
struct MatchResult {
    winner: String,
    loser: String,
    series: SeriesKind,
}

#[derive(serde::Deserialize, std::marker::Copy, std::clone::Clone)]
struct KBracket {
    start: u32,
    k: f32
}

#[derive(serde::Deserialize, std::clone::Clone)]
struct Configuration {
    bo1_score: f32,
    bo3_score: f32,
    bo5_score: f32,
    k_brackets: Vec<KBracket>
}

fn get_series_win_weight_from_config(configuration: Configuration) -> impl Fn(SeriesKind) -> f32 {
    let series_fn = move |series| {
        match series {
            SeriesKind::Bo1 => configuration.bo1_score,
            SeriesKind::Bo3 => configuration.bo3_score,
            SeriesKind::Bo5 => configuration.bo5_score,
        }
    };

    return series_fn;
}

fn get_expected_probabilities(rating1: f32, rating2: f32) -> (f32, f32) {
    let p1 = 1f32 / (1f32 + f32::powf(10f32, (rating2 - rating1) / 400f32));
    let p2 = 1f32 / (1f32 + f32::powf(10f32, (rating1 - rating2) / 400f32));

    return (p1, p2);
}

fn scaling_for_rating(rating: f32, k_brackets: &Vec<KBracket>) -> Option<f32> {
    let mut k_brackets_sorted: Vec<KBracket> = k_brackets.clone();
    k_brackets_sorted.sort_by_key(|bracket| bracket.start);

    for bracket in k_brackets_sorted.iter() {
        if rating >= bracket.start as f32 {
            return Some(bracket.k)
        }
    };

   None 
}

fn combine_ratings(rating1: f32, rating2: f32) -> f32 {
    (rating1 + rating2) / 2f32
}

fn scaling_for_rating_difference(rating1: f32, rating2: f32, k_brackets: &Vec<KBracket>) -> Option<f32> {
    let bracket_rating = combine_ratings(rating1, rating2);
    scaling_for_rating(bracket_rating, k_brackets)
}

fn adjust_ratings(
    rating1: f32,
    rating2: f32,
    k: f32,
    actual_score1: f32,
    actual_score2: f32,
) -> (f32, f32) {
    let expected_probabilities = get_expected_probabilities(rating1, rating2);

    let new_rating1 = rating1 + k * (actual_score1 - expected_probabilities.0);
    let new_rating2 = rating2 + k * (actual_score2 - expected_probabilities.1);

    return (new_rating1, new_rating2);
}

fn apply_match_result(result: &MatchResult, standings: &Standings, series_win_weight:  &impl Fn(SeriesKind) -> f32, k_brackets: &Vec<KBracket>) -> Option<Standings> {
    let winner_rating = standings.get(&result.winner)?;
    let loser_rating = standings.get(&result.loser)?;

    let mut new_standings = standings.clone();
    let new_ratings = adjust_ratings(
        *winner_rating,
        *loser_rating,
        scaling_for_rating_difference(*winner_rating, *loser_rating, k_brackets)?,
        series_win_weight(result.series),
        0f32,
    );
    new_standings.insert(result.winner.clone(), new_ratings.0);
    new_standings.insert(result.loser.clone(), new_ratings.1);

    return Some(new_standings);
}

fn apply_match_results(results: &Vec<MatchResult>, standings: &Standings, k_brackets: &Vec<KBracket>, series_win_weight: &impl Fn(SeriesKind) -> f32) -> Option<Standings> {
    results
        .iter()
        .fold(Some(standings.clone()), |acc, result| match acc {
            Some(standing) => apply_match_result(result, &standing, series_win_weight, k_brackets),
            None => None,
        })
}

fn parse_type_from_path<'a, T>(path: &Path) -> Result<T, Box<dyn Error>> 
where
    for<'de> T: serde::Deserialize<'de> + 'a
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

fn parse_standings_from_path(path: &Path) -> Result<Standings, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let standings = serde_json::from_reader(reader)?;
    Ok(standings)
}

fn parse_match_results_from_path(path: &Path) -> Result<Vec<MatchResult>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let results = serde_json::from_reader(reader)?;
    Ok(results)
}

fn write_standings_to_path(path: &Path, standings: &Standings) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;
    let standings_string = serde_json::to_string_pretty(standings)?;
    file.write_all(standings_string.as_bytes())?;

    return Ok(());
}

fn main() {
    let matches = App::new("ELO System")
                          .version("1.0")
                          .author("Steven Pham")
                          .about("Calculates evolution of team elo after match sets")
                          .arg(Arg::with_name("config")
                              .short("c")
                              .long("config")
                              .value_name("FILE")
                              .help("Path to config file, default is `config.toml`")
                              .takes_value(true))
                          .arg(Arg::with_name("standings")
                              .short("s")
                              .long("standings")
                              .value_name("FILE")
                              .help("Path to standings file")
                              .takes_value(true)
                              .required(true))
                          .arg(Arg::with_name("matches")
                              .short("m")
                              .long("matches")
                              .value_name("FILE")
                              .help("Path to matches file")
                              .takes_value(true)
                              .required(true))
                          .arg(Arg::with_name("output")
                              .short("o")
                              .long("output")
                              .value_name("FILE")
                              .help("Path to output standings")
                              .takes_value(true)
                              .required(true)).get_matches();

    let standings_path = matches.value_of("standings").unwrap();
    let matches_path = matches.value_of("matches").unwrap();
    let output_path = matches.value_of("output").unwrap();
    let config_path = matches.value_of("config").unwrap_or("config.json");

    let standings = match parse_standings_from_path(Path::new(standings_path)) {
        Ok(v) => v,
        Err(error) => panic!("Problem reading standings: {:?}", error),
    };

    let matches = match parse_match_results_from_path(Path::new(matches_path)) {
        Ok(v) => v,
        Err(error) => panic!("Problem reading match results: {:?}", error),
    };

    let config = match parse_type_from_path::<Configuration>(Path::new(config_path)) {
        Ok(v) => v,
        Err(error) => panic!("Problem reading config results: {:?}", error),
    };

    let series_win_weight = get_series_win_weight_from_config(config.clone());

    match write_standings_to_path(Path::new(output_path), &apply_match_results(&matches, &standings, &config.k_brackets, &series_win_weight).unwrap()) {
        Ok(v) => v,
        Err(error) => panic!("Problem writing standings: {:?}", error)
    };
}
