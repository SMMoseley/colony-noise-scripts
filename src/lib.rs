#[macro_use]
extern crate log;
use itertools::Itertools;
use std::{collections::HashMap, iter};
use thiserror::Error as ThisError;

mod stimulus;
use stimulus::{AttributeLabel, Stimulus, StimulusAttribute};

mod choices;
pub use choices::CorrectChoices;

mod decide;
pub use decide::{DecideConfig, Response, StimulusConfig};

mod experiment;
pub use experiment::Experiment;

pub type ConfigWithParams<'a> = (DecideConfig, HashMap<AttributeLabel, StimulusAttribute>);
pub fn make_configs<'a, 'b>(
    experiment: &'a Experiment,
    correct_choices: &'b CorrectChoices,
) -> Result<Vec<ConfigWithParams<'a>>, Error> {
    let inverted_choices = correct_choices.inverted();
    let format_arguments = experiment.named_args()?;
    trace!("named args: {:?}", format_arguments);
    info!("Starting config iteration");
    debug_assert!(!experiment.stimuli_subsets().is_empty());
    let mut attributes = experiment
        .attribute_labels()
        .filter_map(|label| {
            if format_arguments.contains(label) {
                experiment
                    .list_variants(label)
                    .map(|x| iter::repeat(label).zip(x.into_iter()))
            } else {
                None
            }
        })
        .multi_cartesian_product();
    itertools::iproduct!(
        experiment.stimuli_subsets().into_iter(),
        iter::once(attributes.next().unwrap_or_else(Vec::new)).chain(attributes),
        vec![true, false]
    )
    .map(|((set_name, set), attributes, inverted)| {
        trace!("item {:?}", attributes);
        let correct = if inverted {
            &inverted_choices
        } else {
            correct_choices
        };
        let stimuli = set
            .into_iter()
            .filter(experiment.make_filter(&attributes.iter()))
            .map(|stim| StimulusConfig::from(stim, correct))
            .collect::<Result<Vec<_>, _>>()?;
        let parameters = experiment.decide_parameters();
        let stimulus_root = experiment.stimulus_root();
        let config = DecideConfig::new(stimuli, stimulus_root, parameters);
        let mut attributes: HashMap<_, _> = attributes
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        attributes.insert(
            AttributeLabel::from("set"),
            StimulusAttribute::from(&set_name[..]),
        );
        attributes.insert(
            AttributeLabel::from("inverted"),
            StimulusAttribute::from(if inverted { "Yes" } else { "No" }),
        );
        Ok((config, attributes))
    })
    .collect()
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("the attribute listed for `decisive_attribute` was not found")]
    DecisiveAttributeNotFound,
    #[error("could not parse format string")]
    Format,
    #[error("could not find stimulus attribute {0} in correct choices file")]
    StimMissingFromCorrectChoices(StimulusAttribute),
    #[error("the list of choices provided in the experiment file should not be empty")]
    EmptyChoices,
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
