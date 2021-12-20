use itertools::Itertools;
use serde::Deserialize;
use serde_value::Value;
use std::path::Path;
use thiserror::Error as ThisError;

mod stimulus;
use stimulus::{StimulusBaseName, StimulusName, StimulusWithGroup};

mod choices;
pub use choices::CorrectChoices;

mod decide;
pub use decide::{DecideConfig, Response, StimulusConfig};

pub type ConfigWithParams = (bool, Option<i32>, usize, DecideConfig);
pub fn make_configs(
    experiment: &Experiment,
    correct_choices: &CorrectChoices,
) -> Result<Vec<ConfigWithParams>, Error> {
    let inverted_choices = correct_choices.inverted();
    itertools::iproduct!(
        experiment.stimuli_subsets().into_iter().enumerate(),
        experiment.groups(),
        vec![true, false]
    )
    .map(|((set_index, set), group, invert)| {
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
        Ok((invert, group, set_index, config))
    })
    .collect()
}

#[derive(Deserialize)]
pub struct Experiment {
    decide: ExperimentConfig,
    scenes: ScenesConfig,
}

impl Experiment {
    pub fn get_name(&self) -> String {
        self.decide.output_config_name.clone()
    }

    pub fn groups(&self) -> Vec<Option<i32>> {
        let groups: Vec<_> = self
            .stimuli()
            .into_iter()
            .filter_map(|stim| stim.group())
            .unique()
            .collect();
        match groups.is_empty() {
            true => vec![None],
            false => groups.into_iter().map(Some).collect(),
        }
    }

    pub fn stimuli(&self) -> Vec<StimulusName> {
        let foregrounds = self
            .scenes
            .foreground
            .iter()
            .cartesian_product(self.scenes.foreground_dbfs.iter().copied());
        let backgrounds = self
            .scenes
            .background
            .iter()
            .cartesian_product(self.scenes.background_dbfs.iter().copied());
        if self.decide.include_background {
            foregrounds
                .cartesian_product(backgrounds)
                .map(StimulusName::from)
                .collect()
        } else {
            foregrounds.map(StimulusName::from).collect()
        }
    }
    pub fn stimuli_subsets(&self) -> Vec<Vec<StimulusWithGroup>> {
        self.decide
            .stimuli_subsets
            .as_ref()
            .unwrap_or(&vec![self.scenes.foreground.clone()])
            .iter()
            .map(|set| {
                self.stimuli()
                    .into_iter()
                    .filter(|name| set.contains(name.foreground()))
                    .map(|name| {
                        let group = name.group();
                        StimulusWithGroup { name, group }
                    })
                    .collect()
            })
            .collect()
    }
}

#[derive(Deserialize)]
struct ExperimentConfig {
    parameters: Value,
    output_config_name: String,
    stimulus_root: Box<Path>,
    choices: (Response, Response),
    stimuli_subsets: Option<Vec<Vec<StimulusBaseName>>>,
    include_background: bool,
}

#[allow(unused)]
#[derive(Deserialize)]
struct ScenesConfig {
    padding: f64,
    gap: f64,
    ramp: f64,
    #[serde(rename = "foreground-dBFS")]
    foreground_dbfs: Vec<i32>,
    #[serde(rename = "background-dBFS")]
    background_dbfs: Vec<i32>,
    foreground: Vec<StimulusBaseName>,
    background: Vec<StimulusBaseName>,
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
