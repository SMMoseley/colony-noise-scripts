use super::{Error, Experiment, Response, Stimulus, StimulusAttribute};
use itertools::Itertools;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::{collections::HashMap, iter};

#[derive(Serialize, Deserialize, Clone)]
pub struct CorrectChoices(HashMap<StimulusAttribute, Response>);

impl CorrectChoices {
    pub fn get(&self, key: &Stimulus) -> Result<&Response, Error> {
        let key = key.decisive_attribute();
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
        let mut choices = experiment.choices();
        choices.shuffle(&mut rng);
        if choices.is_empty() {
            return Err(Error::EmptyChoices);
        }
        let all_options = experiment
            .list_variants(experiment.decisive_attribute())
            .unwrap();
        let stimuli_per_response = all_options.len() / choices.len();
        let remainder = all_options.len() % choices.len();
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
            all_options
                .into_iter()
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
                    a: b
                stimulus_root: /
                name_format: config
                choices:
                    - peck_left
                    - peck_right
            stimuli:
                format: \"{foreground}\"
                decisive_attribute: foreground
                foreground:
                    values:
                        - a
                        - b
                        - c
                        - d
                        - e
                        - f
        ",
            )
            .unwrap()
        };
    }

    #[test]
    fn random_correctchoices() {
        let exp = exp!();
        let correct = CorrectChoices::random(&exp).unwrap();
        let n_stimuli = exp.list_variants(exp.decisive_attribute()).unwrap().len();
        let n_choices = 2;
        let by_response = |resp| correct.0.values().filter(|&&x| x == resp).count();
        let left_count = by_response(Response::peck_left);
        let right_count = by_response(Response::peck_right);
        assert!(left_count <= n_stimuli / n_choices + 1);
        assert!(right_count <= n_stimuli / n_choices + 1);
    }
}
