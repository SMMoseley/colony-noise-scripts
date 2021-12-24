#[macro_use]
extern crate log;
extern crate pretty_env_logger;
use std::{fs::File, io};
#[macro_use]
extern crate clap;
use anyhow::{Context, Result};
use decide_config::{CorrectChoices, Error, Experiment};
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
    (@arg experiment: <EXPERIMENT_YML> "yaml file containing stimuli, responses, and parameters")
    (@arg correct: -c --("correct-choices") <CORRECT_YML> !required correct_choices_help)
    )
    .get_matches();
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
