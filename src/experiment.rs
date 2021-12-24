use super::{
    stimulus::StimuliConfig, AttributeLabel, Error, Response, Stimulus, StimulusAttribute,
};
use dynfmt::{curly::SimpleCurlyFormat, Format};
use serde::Deserialize;
use serde_value::Value;
use std::{collections::HashMap, convert::TryFrom, path::Path};

#[derive(Deserialize)]
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
                    .ok_or(Error::Format)
                    .map(|label| label.clone())
            })
            .collect()
    }

    pub fn name_format(&self) -> String {
        self.decide.name_format.clone()
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

    pub fn list_variants(&self, label: &AttributeLabel) -> Option<Vec<&StimulusAttribute>> {
        self.stimuli.list_variants(label)
    }

    pub fn make_filter<'a, I>(&self, attributes: &'a I) -> impl FnMut(&Stimulus) -> bool + 'a
    where
        I: IntoIterator<Item = &'a (&'a AttributeLabel, &'a StimulusAttribute)> + Clone,
    {
        move |stimulus: &Stimulus| {
            let mut attributes = attributes.clone().into_iter();
            attributes.all(|x| stimulus.matches(x))
        }
    }

    pub fn decide_parameters(&self) -> Value {
        self.decide.parameters.clone()
    }

    pub fn stimulus_root(&self) -> Box<Path> {
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
struct PermissiveExperimentConfig {
    parameters: Value,
    name_format: String,
    stimulus_root: Box<Path>,
    choices: (Response, Response),
    stimuli_subsets: Option<HashMap<String, Vec<StimulusAttribute>>>,
}

#[derive(Deserialize)]
#[serde(try_from = "PermissiveExperimentConfig")]
pub struct ExperimentConfig {
    pub parameters: Value,
    pub name_format: String,
    pub stimulus_root: Box<Path>,
    pub choices: (Response, Response),
    pub stimuli_subsets: Option<HashMap<String, Vec<StimulusAttribute>>>,
}

impl ExperimentConfig {
    pub fn named_args(&self) -> Result<Vec<&str>, Error> {
        SimpleCurlyFormat
            .iter_args(&self.name_format)
            .map_err(|_| Error::Format)?
            .map(|arg_spec| {
                trace!("found named arg");
                let arg_spec = arg_spec.map_err(|_| Error::Format)?;
                Ok(&self.name_format[arg_spec.start() + 1..arg_spec.end() - 1])
            })
            .collect()
    }
}

impl TryFrom<PermissiveExperimentConfig> for ExperimentConfig {
    type Error = Error;

    fn try_from(exp: PermissiveExperimentConfig) -> Result<Self, Self::Error> {
        //let foreground: HashSet<_> = exp.scenes.foreground.iter().collect();
        //let set: HashSet<_> = set.into_iter().collect();
        //if !set.is_subset(&foreground) {
        //    panic!("set should be a subset of foreground");
        //}
        Ok(ExperimentConfig {
            parameters: exp.parameters,
            name_format: exp.name_format,
            stimulus_root: exp.stimulus_root,
            choices: exp.choices,
            stimuli_subsets: exp.stimuli_subsets,
        })
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
