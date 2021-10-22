use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use serde_value::Value;
use serde_with::skip_serializing_none;
use std::{collections::HashMap, iter, path::Path};
use thiserror::Error as ThisError;

#[derive(Serialize)]
pub struct DecideConfig {
    parameters: Value,
    stimulus_root: Box<Path>,
    stimuli: Vec<StimulusConfig>,
}

impl DecideConfig {
    pub fn from(
        experiment: Experiment,
        correct_choices: CorrectChoices,
        invert: bool,
    ) -> Result<Self, Error> {
        let parameters = experiment.config.parameters;
        let stimulus_root = experiment.config.stimulus_root;
        let keys = experiment.config.keys;
        let choices = experiment.config.choices;
        let stimuli = experiment
            .stimuli
            .into_iter()
            .map(|name| {
                let responses = keys
                    .iter()
                    .chain(iter::once(&Response::timeout))
                    .map(|&response| {
                        let correct_response = *correct_choices.get(&name)?;
                        let response_meaning = if choices.contains(&response) {
                            if (response == correct_response) ^ invert {
                                ResponseMeaning::Correct
                            } else {
                                ResponseMeaning::Incorrect
                            }
                        } else {
                            ResponseMeaning::Incorrect
                        };
                        Ok((response, response_meaning.into()))
                    })
                    .collect::<Result<_, _>>()?;
                Ok(StimulusConfig {
                    name,
                    frequency: 1,
                    category: Some("no_cue_lights".into()),
                    cue_resp: None,
                    responses,
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(DecideConfig {
            parameters,
            stimulus_root,
            stimuli,
        })
    }
}

#[skip_serializing_none]
#[derive(Serialize)]
struct StimulusConfig {
    name: StimulusName,
    frequency: u32,
    responses: HashMap<Response, Outcome>,
    category: Option<String>,
    cue_resp: Option<Vec<Light>>,
}

#[derive(Deserialize, Serialize, Hash, PartialEq, Eq, Clone, Copy)]
enum Response {
    peck_left,
    peck_center,
    peck_right,
    timeout,
}

enum ResponseMeaning {
    Correct,
    Incorrect,
    Neutral,
}

#[skip_serializing_none]
#[derive(Serialize)]
struct Outcome {
    p_reward: Option<f64>,
    p_punish: Option<f64>,
    correct: bool,
}

impl From<ResponseMeaning> for Outcome {
    fn from(response: ResponseMeaning) -> Self {
        match response {
            ResponseMeaning::Correct => Outcome {
                p_reward: Some(1.0),
                p_punish: None,
                correct: true,
            },
            ResponseMeaning::Incorrect => Outcome {
                p_punish: Some(1.0),
                p_reward: None,
                correct: false,
            },
            ResponseMeaning::Neutral => Outcome {
                p_punish: None,
                p_reward: None,
                correct: false,
            },
        }
    }
}

#[derive(Serialize, Hash, PartialEq, Eq)]
enum Light {
    left_blue,
    center_blue,
    right_blue,
}

#[derive(Deserialize)]
pub struct Experiment {
    config: ExperimentConfig,
    stimuli: Vec<StimulusName>,
}

impl Experiment {
    pub fn get_name(&self) -> String {
        self.config.output_config_name.clone()
    }
}

#[derive(Deserialize)]
struct ExperimentConfig {
    parameters: Value,
    output_config_name: String,
    stimulus_root: Box<Path>,
    keys: Vec<Response>,
    choices: Vec<Response>,
}

#[derive(Serialize, Deserialize)]
pub struct CorrectChoices(HashMap<StimulusName, Response>);

impl CorrectChoices {
    fn get(&self, key: &StimulusName) -> Result<&Response, Error> {
        if let Some(r) = self.0.get(key) {
            Ok(r)
        } else {
            let matching_keys = self.0.keys().filter(|k| key.starts_with(k));
            match matching_keys.count() {
                0 => Err(Error::StimMissingFromCorrectChoices(key.clone())),
                1 => Ok(self
                    .0
                    .iter()
                    .filter(|(k, _)| key.starts_with(k))
                    .next()
                    .unwrap()
                    .1),
                _ => Err(Error::AmbiguousPrefix(key.clone())),
            }
        }
    }
    pub fn random(experiment: &Experiment) -> Result<Self, Error> {
        let mut rng = thread_rng();
        Ok(CorrectChoices(
            experiment
                .stimuli
                .iter()
                .map(|s| {
                    Ok((
                        s.clone(),
                        *experiment
                            .config
                            .choices
                            .choose(&mut rng)
                            .ok_or_else(|| Error::EmptyChoices)?,
                    ))
                })
                .collect::<Result<_, _>>()?,
        ))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Hash, Eq, Clone, Debug)]
pub struct StimulusName(String);

impl StimulusName {
    pub fn starts_with(&self, pat: &StimulusName) -> bool {
        self.0.starts_with(&pat.0)
    }
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Could not find stimulus {0:?} in correct choices file")]
    StimMissingFromCorrectChoices(StimulusName),
    #[error("The stimulus {0:?} matched multiple values in the correct choices file")]
    AmbiguousPrefix(StimulusName),
    #[error("The list of choices provided in the experiment file should not be empty")]
    EmptyChoices,
}
