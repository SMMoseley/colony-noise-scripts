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
        let all_values = experiment
            .list_attribute_values(experiment.decisive_attribute())
            .unwrap();
        Self::random_with_choices(experiment.choices(), all_values)
    }

    fn random_with_choices<'a, I>(mut choices: Vec<Response>, all_values: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = &'a StimulusAttribute>,
    {
        let all_values: Vec<_> = all_values.into_iter().collect();
        if choices.is_empty() {
            return Err(Error::EmptyChoices);
        }
        let mut rng = thread_rng();
        choices.shuffle(&mut rng);
        let stimuli_per_response = all_values.len() / choices.len();
        let remainder = all_values.len() % choices.len();
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
            all_values
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

    #[test]
    fn random_correctchoices() {
        let choices = vec![Response::PeckLeft, Response::PeckRight];
        let all_values: Vec<_> = vec!["a", "b", "c", "d"]
            .into_iter()
            .map(StimulusAttribute::from)
            .collect();
        let correct = CorrectChoices::random_with_choices(choices, all_values.iter()).unwrap();
        let n_stimuli = 4;
        let n_choices = 2;
        let by_response = |resp| correct.0.values().filter(|&&x| x == resp).count();
        let left_count = by_response(Response::PeckLeft);
        let right_count = by_response(Response::PeckRight);
        assert!(left_count <= n_stimuli / n_choices + 1);
        assert!(right_count <= n_stimuli / n_choices + 1);
    }
}
