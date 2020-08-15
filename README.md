# Elo

Elo implementation written in Rust with bracket configurable K values and differing weights for Best of N series.

## Building
This project uses Rust nightly, Rust can be installed with [rustup](https://rustup.rs/).

```
> git clone https://github.com/Eliasin/elo.git
> cd elo
> cargo build --release
```

## Usage
```
USAGE:
    elo [OPTIONS] --matches <FILE> --output <FILE> --standings <FILE>
```

`matches` should be the path to a file containing a JSON representing a list of
```
{
    winner: String,
    loser: String,
    series: SeriesKind
}
```
where `winner` and `loser` are the names of the winning and losing teams
where `SeriesKind` can be the string `"Bo1"`, `"Bo3"` or `"Bo5"`

`standings` should be the path to a file containing a JSON representing team standings as keys from name to rating
```
{
	"team_name1": number,	
	"team_name2": number,
	...
}
```

where there can be any number of team, elo pairs

## Configuration
The configuration file determines the weights for Best of N series and determines the K values for different elo brackets. It is in `config.json` by default but this can be overriden with the `--config` or `-c` flag.

The configuration file should be a JSON representing
```
{
	"bo1_score": number,
	"bo3_score": number,
	"bo5_score": number,
	"k_brackets": [
		{
			"start": number,
			"k": number
		},
		{
			"start": number,
			"k": number
		},
		...
	]
}
```
