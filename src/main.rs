use std::fs::File;
#[macro_use]
extern crate clap;
use clap::{Arg, App};
use anyhow::Result;
use serde_yaml;
use serde_json;
use decide_config::{Experiment, DecideConfig};

fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("Generate config files for chorus noise 2 alternative choice experiment")
        .arg(Arg::with_name("experiment")
             .short("e")
             .long("experiment-file")
             .value_name("EXPERIMENT")
             .help("yaml file containing a list of stimuli and parameters")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("correct")
             .short("c")
             .long("correct-choices-file")
             .value_name("CORRECTCHOICES")
             .help("yaml file containing the correct choice for each stimulus")
             .takes_value(true))
        .arg(Arg::with_name("invert")
             .short("i")
             .long("invert-answers")
             .help("whether to flip correct keys for each stimulus"))
        .get_matches();
    let experiment: Experiment = serde_yaml::from_reader(File::open(matches.value_of("experiment").unwrap())?)?;
    let invert = matches.is_present("invert");
    let config_name = experiment.get_name() + if invert {"-inverted"} else {""} + ".json";
    let correct_choices = serde_yaml::from_reader(File::open(matches.value_of("correct").unwrap())?)?;
    let decide_config = DecideConfig::from(experiment, correct_choices, invert)?;
    let config_file = File::create(config_name)?;
    serde_json::to_writer_pretty(config_file, &decide_config)?;
    Ok(())
}
