use super::{Error, Experiment, Response, StimulusBaseName, StimulusName};
use itertools::Itertools;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::{collections::HashMap, iter};

#[derive(Serialize, Deserialize, Clone)]
pub struct CorrectChoices(HashMap<StimulusBaseName, Response>);

impl CorrectChoices {
    pub fn get(&self, key: &StimulusName) -> Result<&Response, Error> {
        let key = key.foreground();
        self.0
            .get(key)
            .ok_or_else(|| Error::StimMissingFromCorrectChoices(key.clone()))
    }

    fn choices(&self) -> [Response; 2] {
        self.0
            .values()
            .unique()
            .cloned()
            .collect::<Vec<_>>()
            .try_into()
            .expect("invalid correct choices")
    }

    pub fn inverted(&self) -> Self {
        CorrectChoices(
            self.0
                .iter()
                .map(|(name, response)| {
                    let choices = self.choices();
                    let inverse_response = if response == &choices[0] {
                        choices[1]
                    } else if response == &choices[1] {
                        choices[0]
                    } else {
                        panic!("CorrectChoices in an invalid state");
                    };
                    (name.clone(), inverse_response)
                })
                .collect(),
        )
    }

    pub fn random(experiment: &Experiment) -> Result<Self, Error> {
        let mut rng = thread_rng();
        let mut choices = vec![experiment.decide.choices.0, experiment.decide.choices.1];
        choices.shuffle(&mut rng);
        if choices.is_empty() {
            return Err(Error::EmptyChoices);
        }
        let stimuli_per_response = experiment.scenes.foreground.len() / choices.len();
        let remainder = experiment.scenes.foreground.len() % choices.len();
        // we create a vector with one response per stimulus,
        // with evenly divided assignment as much as possible
        let mut matched_choices: Vec<Response> = choices
            .iter()
            .map(|&c| iter::repeat(c).take(stimuli_per_response))
            .flatten()
            .chain(choices.iter().take(remainder).copied())
            .collect();
        matched_choices.shuffle(&mut rng);
        Ok(CorrectChoices(
            experiment
                .scenes
                .foreground
                .iter()
                .cloned()
                .zip(matched_choices)
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! exp {
        () => {
            serde_yaml::from_str::<Experiment>(
                "
            decide:
                parameters:
                output_config_name: config
                stimulus_root: /
                choices:
                    - peck_left
                    - peck_right
                include_background: false
            scenes:
                padding: 0
                gap: 0
                ramp: 0
                foreground:
                    - a
                    - b
                    - c
                    - d
                    - e
                    - f
                foreground-dBFS: []
                background: []
                background-dBFS: []
        ",
            )
            .unwrap()
        };
    }

    #[test]
    fn random_correctchoices() {
        let exp = exp!();
        let correct = CorrectChoices::random(&exp).unwrap();
        let n_stimuli = exp.scenes.foreground.len();
        let n_choices = 2;
        let by_response = |resp| correct.0.values().filter(|&&x| x == resp).count();
        let left_count = by_response(Response::peck_left);
        let right_count = by_response(Response::peck_right);
        assert!(left_count <= n_stimuli / n_choices + 1);
        assert!(right_count <= n_stimuli / n_choices + 1);
    }
}
