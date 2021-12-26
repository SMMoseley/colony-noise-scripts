#[macro_use]
extern crate log;
extern crate pretty_env_logger;
use std::{fs::File, io};
#[macro_use]
extern crate clap;
use anyhow::{Context, Result};
use clap::ArgMatches;
use decide_config::{CorrectChoices, DecideConfig, Error, Experiment};
use dynfmt::{Format, SimpleCurlyFormat};

const DEFAULT_CORRECT_CHOICES_FILE: &str = "correct_choices.yml";

fn main() -> Result<()> {
    pretty_env_logger::init();
    let correct_choices_help = &format!(
        "name for file with correct response for each stimulus [default: {}]",
        DEFAULT_CORRECT_CHOICES_FILE
    );
    let matches = clap_app!(
    @app (app_from_crate!())
    (@arg experiment: [EXPERIMENT_YML] "yaml file containing stimuli, responses, and parameters")
    (@arg correct: -c --("correct-choices") [CORRECT_YML] correct_choices_help)
    (@subcommand diff =>
        (about: "compare two decide-config JSON output files")
        (@arg file1: <FILE1>)
        (@arg file2: <FILE2>)
    )
    )
    .get_matches();

    match matches.subcommand() {
        ("diff", Some(matches)) => config_diff(matches),
        _ => generate_configs(matches),
    }
}

fn generate_configs(matches: ArgMatches) -> Result<()> {
    let experiment: Experiment = serde_yaml::from_reader(
        File::open(matches.value_of("experiment").unwrap())
            .context("could not open experiment file")?,
    )
    .context("could not parse experiment file")?;
    let correct_choices_name = matches
        .value_of("correct")
        .unwrap_or(DEFAULT_CORRECT_CHOICES_FILE);
    let correct_choices = match File::open(correct_choices_name) {
        Ok(file) => {
            serde_yaml::from_reader(file).context("could not parse correct choices file")?
        }
        Err(e) => {
            if let io::ErrorKind::NotFound = e.kind() {
                let choices = CorrectChoices::random(&experiment)?;
                let file = File::create(correct_choices_name)
                    .context("could not create correct choices file")?;
                serde_yaml::to_writer(file, &choices)
                    .context("could not write correct choices file")?;
                Ok(choices)
            } else {
                Err(e)
            }
        }?,
    };
    for (config, attributes) in decide_config::make_configs(&experiment, &correct_choices)? {
        let format_str = experiment.name_format() + "-set{set}-inverted{inverted}.json";
        trace!("format string: {}", format_str);
        trace!("attributes: {:?}", attributes);
        let formatted_name = SimpleCurlyFormat
            .format(&format_str, attributes)
            .map_err(|_| Error::Format)?
            .into_owned();
        config.to_json(formatted_name)?;
    }
    Ok(())
}

fn config_diff(matches: &ArgMatches) -> Result<()> {
    let file1_name = matches.value_of("file1").unwrap();
    let file2_name = matches.value_of("file2").unwrap();
    let file1: DecideConfig = serde_json::from_reader(File::open(file1_name)?)
        .with_context(|| format!("could not parse {}", file1_name))?;
    let file2: DecideConfig = serde_json::from_reader(File::open(file2_name)?)
        .with_context(|| format!("could not parse {}", file2_name))?;
    if file1 == file2 {
        std::process::exit(0)
    } else {
        eprintln!("Files differ!");
        std::process::exit(1)
    }
}
