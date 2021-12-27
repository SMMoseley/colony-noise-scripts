use super::{CorrectChoices, Error, Stimulus};
use anyhow::Context;
use fixed::traits::ToFixed;
use fixed::types::I20F12;
use fixed_macro::fixed;
use serde::{Deserialize, Serialize};
use serde_value::Value;
use serde_with::skip_serializing_none;
use std::{
    collections::{BTreeMap, HashSet},
    fs::File,
    path::PathBuf,
};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct DecideConfig {
    parameters: Value,
    stimulus_root: PathBuf,
    stimuli: HashSet<StimulusConfig>,
}

impl DecideConfig {
    pub fn new<I>(stimuli: I, stimulus_root: PathBuf, parameters: Value) -> Self
    where
        I: IntoIterator<Item = StimulusConfig>,
    {
        let stimuli = stimuli.into_iter().collect();
        DecideConfig {
            stimuli,
            stimulus_root,
            parameters,
        }
    }

    pub fn to_json(&self, config_name: String) -> anyhow::Result<()> {
        let config_file = File::create(&config_name)
            .with_context(|| format!("could not create config `{}`", config_name))?;
        serde_json::to_writer_pretty(config_file, &self)
            .with_context(|| format!("could not write config `{}`", config_name))?;
        Ok(())
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct StimulusConfig {
    name: String,
    frequency: u32,
    responses: BTreeMap<Response, Outcome>,
    category: Option<String>,
}

impl StimulusConfig {
    pub fn from(name: Stimulus<'_>, correct_choices: &CorrectChoices) -> Result<Self, Error> {
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
            name: name.into(),
            frequency: 1,
            category: Some("no_cue_lights".into()),
            responses,
        })
    }
}

#[derive(
    Deserialize, Serialize, PartialEq, Eq, Clone, Copy, EnumIter, PartialOrd, Ord, Hash, Debug,
)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    PeckLeft,
    PeckCenter,
    PeckRight,
    Timeout,
}

#[allow(unused)]
enum ResponseMeaning {
    Correct,
    Incorrect,
    Neutral,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(into = "f64")]
#[serde(from = "f64")]
struct Decimal(I20F12);

impl From<f64> for Decimal {
    fn from(x: f64) -> Self {
        Decimal(x.to_fixed())
    }
}

impl From<Decimal> for f64 {
    fn from(x: Decimal) -> Self {
        Self::from(x.0)
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Outcome {
    p_reward: Option<Decimal>,
    p_punish: Option<Decimal>,
    correct: bool,
}

impl From<ResponseMeaning> for Outcome {
    fn from(response: ResponseMeaning) -> Self {
        match response {
            ResponseMeaning::Correct => Outcome {
                p_reward: Some(Decimal(fixed!(1.0: I20F12))),
                p_punish: None,
                correct: true,
            },
            ResponseMeaning::Incorrect => Outcome {
                p_punish: Some(Decimal(fixed!(1.0: I20F12))),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_decimal() {
        let f: Decimal = serde_yaml::from_str("1.0").unwrap();
        assert_eq!(f64::from(f), 1.0);
    }

    #[test]
    fn deserialize_outcome() {
        let _out: Outcome = serde_json::from_str(
            "
        {
            \"p_reward\": 1.0,
            \"p_punish\": 0.0,
            \"correct\": false
        }
        ",
        )
        .unwrap();
    }

    #[test]
    fn serialize_decimal() {
        let f: Decimal = Decimal(fixed!(1.0: I20F12));
        assert_eq!(serde_json::to_string(&f).unwrap(), "1.0");
    }
}
