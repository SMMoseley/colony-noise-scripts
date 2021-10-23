use std::{fs::File, io};
#[macro_use]
extern crate clap;
use anyhow::Result;
use clap::{App, Arg};
use decide_config::{CorrectChoices, DecideConfig, Experiment};

fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("Generate config files for chorus noise 2 alternative choice experiment")
        .arg(
            Arg::with_name("experiment")
                .value_name("EXPERIMENT")
                .help("yaml file containing a list of stimuli and parameters")
                .required(true)
                .index(1)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("correct")
                .short("c")
                .long("correct-choices-file")
                .value_name("CORRECTCHOICES")
                .default_value("correct_choices.yml")
                .help("yaml file containing the correct choice for each stimulus")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("no-invert")
                .short("i")
                .long("no-inverted-config")
                .help("skip generating an inverted answers config"),
        )
        .get_matches();
    let experiment: Experiment =
        serde_yaml::from_reader(File::open(matches.value_of("experiment").unwrap())?)?;
    let correct_choices = match File::open(matches.value_of("correct").unwrap()) {
        Ok(file) => serde_yaml::from_reader(file)?,
        Err(e) => {
            if let io::ErrorKind::NotFound = e.kind() {
                let choices = CorrectChoices::random(&experiment)?;
                let file = File::create(matches.value_of("correct").unwrap())?;
                serde_yaml::to_writer(file, &choices)?;
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
        DecideConfig::from(&experiment, &correct_choices, true, group)?.to_json(format!(
            "{}{}.json",
            experiment.get_name(),
            segmented
        ))?;
        if !matches.is_present("no-invert") {
            DecideConfig::from(&experiment, &correct_choices, true, group)?.to_json(format!(
                "{}-inverted{}.json",
                experiment.get_name(),
                segmented
            ))?;
        }
    }
    Ok(())
}
