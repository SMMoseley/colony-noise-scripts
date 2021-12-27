use super::{
    stimulus::StimuliConfig, AttributeLabel, Error, Response, Stimulus, StimulusAttribute,
};
use dynfmt::{curly::SimpleCurlyFormat, Format};
use serde::Deserialize;
use serde_value::Value;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    path::PathBuf,
};

#[derive(Deserialize)]
#[serde(try_from = "UnvalidatedExperiment")]
pub struct Experiment {
    decide: ExperimentConfig,
    stimuli: StimuliConfig,
}

impl Experiment {
    pub fn stimuli(&self) -> Vec<Stimulus<'_>> {
        self.stimuli.stimuli()
    }

    pub fn named_args(&self) -> Result<Vec<AttributeLabel>, Error> {
        let args = self.decide.named_args()?;
        args.into_iter()
            .map(|name| {
                self.stimuli
                    .label_by_str(name)
                    .ok_or_else(|| Error::UnknownAttributeInNameFormat(String::from(name)))
                    .map(|label| label.clone())
            })
            .collect()
    }

    pub fn name_format(&self) -> &str {
        &self.decide.name_format
    }

    pub fn stimuli_subsets(&self) -> Vec<(String, Vec<Stimulus<'_>>)> {
        self.decide
            .stimuli_subsets
            .as_ref()
            .map(|subsets| {
                subsets
                    .iter()
                    .map(|(name, attribute_set)| {
                        let set = self
                            .stimuli()
                            .into_iter()
                            .filter(|name| attribute_set.contains(name.decisive_attribute()))
                            .collect();
                        (name.clone(), set)
                    })
                    .collect()
            })
            .unwrap_or_else(|| vec![(String::from("All"), self.stimuli())])
    }

    pub fn attribute_labels(&self) -> impl Iterator<Item = &AttributeLabel> {
        self.stimuli.attribute_labels()
    }

    pub fn list_attribute_values(&self, label: &AttributeLabel) -> Option<Vec<&StimulusAttribute>> {
        self.stimuli.list_values(label)
    }

    pub fn decide_parameters(&self) -> Value {
        self.decide.parameters.clone()
    }

    pub fn stimulus_root(&self) -> PathBuf {
        self.decide.stimulus_root.clone()
    }

    pub fn choices(&self) -> Vec<Response> {
        vec![self.decide.choices.0, self.decide.choices.1]
    }

    pub fn decisive_attribute(&self) -> &AttributeLabel {
        self.stimuli.decisive_attribute()
    }
}

#[derive(Deserialize)]
pub struct ExperimentConfig {
    pub parameters: Value,
    pub name_format: String,
    pub stimulus_root: PathBuf,
    pub choices: (Response, Response),
    pub stimuli_subsets: Option<HashMap<String, Vec<StimulusAttribute>>>,
}

impl ExperimentConfig {
    pub fn named_args(&self) -> Result<Vec<&str>, Error> {
        SimpleCurlyFormat
            .iter_args(&self.name_format)
            .map_err(|_| Error::Format)?
            .map(|arg_spec| {
                let arg_spec = arg_spec.map_err(|_| Error::Format)?;
                Ok(&self.name_format[arg_spec.start() + 1..arg_spec.end() - 1])
            })
            .collect()
    }
}

#[derive(Deserialize)]
struct UnvalidatedExperiment {
    decide: ExperimentConfig,
    stimuli: StimuliConfig,
}

impl TryFrom<UnvalidatedExperiment> for Experiment {
    type Error = Error;

    fn try_from(
        UnvalidatedExperiment { decide, stimuli }: UnvalidatedExperiment,
    ) -> Result<Self, Self::Error> {
        let all_values: HashSet<_> = stimuli
            .list_values(stimuli.decisive_attribute())
            .ok_or(Error::DecisiveAttributeNotFound)?
            .into_iter()
            .collect();
        if let Some(stimuli_subsets) = decide.stimuli_subsets.as_ref() {
            for (name, subset) in stimuli_subsets.iter() {
                let subset: HashSet<_> = subset.iter().collect();
                if !subset.is_subset(&all_values) {
                    return Err(Error::NotASubset(name.clone()));
                }
            }
        }
        Ok(Experiment { decide, stimuli })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_args() {
        let exp: ExperimentConfig = serde_yaml::from_str(
            "
            parameters:
            name_format: \"{a}{b}{c}\"
            stimulus_root: /
            choices:
                - peck_left
                - peck_right
            ",
        )
        .unwrap();
        let named_args = exp.named_args().unwrap();
        assert_eq!(named_args, vec!["a", "b", "c"]);
    }
}
