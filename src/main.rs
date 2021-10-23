use std::{fs::File, io};
#[macro_use]
extern crate clap;
use anyhow::{Context, Result};
use decide_config::{CorrectChoices, DecideConfig, Experiment};

fn main() -> Result<()> {
    let matches = clap_app!(
        @app (app_from_crate!())
        (@arg experiment: <EXPERIMENT_YML> "yaml file containing stimuli, responses, and parameters")
        (@arg correct: -c --("correct-choices") <CORRECT_YML>
         !required "name for file with correct response for each stimulus")
        (@arg no_invert: -i --("no-inverted-config") "skip generating an inverted answers config")
    ).get_matches();
    let experiment: Experiment = serde_yaml::from_reader(
        File::open(matches.value_of("experiment").unwrap())
            .context("could not open experiment file")?,
    )
    .context("could not parse experiment file")?;
    let correct_choices_name = matches.value_of("correct").unwrap_or("correct_choices.yml");
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
    for group in experiment.groups() {
        let segmented = group
            .map(|g| format!("-segmented{}", g))
            .unwrap_or_else(|| "".into());
        DecideConfig::from(&experiment, &correct_choices, false, group)?.to_json(format!(
            "{}{}.json",
            experiment.get_name(),
            segmented
        ))?;
        if !matches.is_present("no_invert") {
            DecideConfig::from(&experiment, &correct_choices, true, group)?.to_json(format!(
                "{}-inverted{}.json",
                experiment.get_name(),
                segmented
            ))?;
        }
    }
    Ok(())
}
