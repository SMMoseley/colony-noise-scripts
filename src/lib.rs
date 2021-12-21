use thiserror::Error as ThisError;

mod stimulus;
use stimulus::{StimulusBaseName, StimulusName};

mod choices;
pub use choices::CorrectChoices;

mod decide;
pub use decide::{DecideConfig, Response, StimulusConfig};

mod experiment;
pub use experiment::Experiment;

pub type ConfigWithParams = (bool, Option<i32>, String, DecideConfig);
pub fn make_configs(
    experiment: &Experiment,
    correct_choices: &CorrectChoices,
) -> Result<Vec<ConfigWithParams>, Error> {
    let inverted_choices = correct_choices.inverted();
    let groups = experiment.groups();
    itertools::iproduct!(
        experiment.stimuli_subsets().into_iter(),
        groups,
        vec![true, false]
    )
    .map(|((set_name, set), group, invert)| {
        let correct = if invert {
            &inverted_choices
        } else {
            correct_choices
        };
        let stimuli = set
            .into_iter()
            .filter(|stim| {
                stim.group
                    .and_then(|sg| group.map(|g| sg <= g))
                    .unwrap_or(true)
            })
            .map(|stim| StimulusConfig::from(stim.name, correct))
            .collect::<Result<Vec<_>, _>>()?;
        let parameters = experiment.decide.parameters.clone();
        let stimulus_root = experiment.decide.stimulus_root.clone();
        let config = DecideConfig::new(stimuli, stimulus_root, parameters);
        Ok((invert, group, set_name, config))
    })
    .collect()
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Could not find stimulus {0} in correct choices file")]
    StimMissingFromCorrectChoices(StimulusBaseName),
    #[error("The list of choices provided in the experiment file should not be empty")]
    EmptyChoices,
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
