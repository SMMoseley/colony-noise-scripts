use super::{CorrectChoices, Error, StimulusName};
use serde::{Deserialize, Serialize};
use serde_value::Value;
use serde_with::skip_serializing_none;
use std::{collections::BTreeMap, fs::File, path::Path};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Serialize)]
pub struct DecideConfig {
    parameters: Value,
    stimulus_root: Box<Path>,
    stimuli: Vec<StimulusConfig>,
}

impl DecideConfig {
    pub fn new(stimuli: Vec<StimulusConfig>, stimulus_root: Box<Path>, parameters: Value) -> Self {
        DecideConfig {
            stimuli,
            stimulus_root,
            parameters,
        }
    }

    pub fn to_json(&self, config_name: String) -> anyhow::Result<()> {
        let config_file = File::create(config_name)?;
        serde_json::to_writer_pretty(config_file, &self)?;
        Ok(())
    }
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct StimulusConfig {
    name: StimulusName,
    frequency: u32,
    responses: BTreeMap<Response, Outcome>,
    category: Option<String>,
    cue_resp: Option<Vec<Light>>,
}

impl StimulusConfig {
    pub fn from(name: StimulusName, correct_choices: &CorrectChoices) -> Result<Self, Error> {
        let responses = Response::iter()
            .map(|response| {
                let correct_response = *correct_choices.get(&name)?;
                let response_meaning = if response == correct_response {
                    ResponseMeaning::Correct
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
    }
}

#[allow(non_camel_case_types)]
#[derive(
    Deserialize, Serialize, PartialEq, Eq, Clone, Copy, EnumIter, PartialOrd, Ord, Hash, Debug,
)]
pub enum Response {
    peck_left,
    peck_center,
    peck_right,
    timeout,
}

#[allow(unused)]
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

#[allow(dead_code, non_camel_case_types)]
#[derive(Serialize, Hash, PartialEq, Eq)]
enum Light {
    left_blue,
    center_blue,
    right_blue,
}
