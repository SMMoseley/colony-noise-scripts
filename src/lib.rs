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
    let mut held_constant_attributes = experiment
        .attribute_labels()
        .filter_map(|label| {
            if format_arguments.contains(label) {
                experiment
                    .list_attribute_values(label)
                    .map(|values| iter::repeat(label).zip(values.into_iter()))
            } else {
                None
            }
        })
        .multi_cartesian_product();
    itertools::iproduct!(
        experiment.stimuli_subsets().into_iter(),
        iter::once(held_constant_attributes.next().unwrap_or_else(Vec::new))
            .chain(held_constant_attributes),
        vec![true, false]
    )
    .map(|((set_name, set), constant_attributes, inverted)| {
        trace!("item {:?}", constant_attributes);
        let correct = if inverted {
            &inverted_choices
        } else {
            correct_choices
        };
        let stimuli = set
            .into_iter()
            .filter(|stimulus| {
                constant_attributes
                    .iter()
                    .all(|attribute| stimulus.matches(attribute))
            })
            .map(|stimulus| StimulusConfig::from(stimulus, correct))
            .collect::<Result<Vec<_>, _>>()?;
        let parameters = experiment.decide_parameters().clone();
        let stimulus_root = experiment.stimulus_root().clone();
        let config = DecideConfig::new(stimuli, stimulus_root, parameters);
        let mut attributes: HashMap<_, _> = constant_attributes
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
    #[error("subset `{0}` contains unlisted attribute values")]
    NotASubset(String),
    #[error("the attribute {0} was used in `name_format`, but was not included under `stimuli`")]
    UnknownAttributeInNameFormat(String),
    #[error("the attribute listed for `decisive_attribute` was not found in `stimuli`")]
    DecisiveAttributeNotFound,
    #[error("an error occured while formating")]
    Format,
    #[error("could not find stimulus attribute {0} in correct choices file")]
    StimMissingFromCorrectChoices(StimulusAttribute),
    #[error("the list of choices provided in the experiment file should not be empty")]
    EmptyChoices,
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct ReadmeDoctests;
