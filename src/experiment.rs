use super::{stimulus::StimulusWithGroup, Error, Response, StimulusBaseName, StimulusName};
use itertools::Itertools;
use serde::Deserialize;
use serde_value::Value;
use std::{collections::HashMap, convert::TryFrom, path::Path};

#[derive(Deserialize)]
pub struct Experiment {
    pub decide: ExperimentConfig,
    pub scenes: ScenesConfig,
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
    pub fn stimuli_subsets(&self) -> Vec<(String, Vec<StimulusWithGroup>)> {
        self.decide
            .stimuli_subsets
            .as_ref()
            .unwrap_or(&{
                let mut default_map = HashMap::new();
                default_map.insert(String::from("all_stimuli"), self.scenes.foreground.clone());
                default_map
            })
            .iter()
            .map(|(name, set)| {
                let set = self
                    .stimuli()
                    .into_iter()
                    .filter(|name| set.contains(name.foreground()))
                    .map(|name| {
                        let group = name.group();
                        StimulusWithGroup { name, group }
                    })
                    .collect();
                (name.clone(), set)
            })
            .collect()
    }
}

#[derive(Deserialize)]
struct PermissiveExperimentConfig {
    parameters: Value,
    output_config_name: String,
    stimulus_root: Box<Path>,
    choices: (Response, Response),
    stimuli_subsets: Option<HashMap<String, Vec<StimulusBaseName>>>,
    include_background: bool,
}

#[derive(Deserialize)]
#[serde(try_from = "PermissiveExperimentConfig")]
pub struct ExperimentConfig {
    pub parameters: Value,
    pub output_config_name: String,
    pub stimulus_root: Box<Path>,
    pub choices: (Response, Response),
    pub stimuli_subsets: Option<HashMap<String, Vec<StimulusBaseName>>>,
    pub include_background: bool,
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
            output_config_name: exp.output_config_name,
            stimulus_root: exp.stimulus_root,
            choices: exp.choices,
            stimuli_subsets: exp.stimuli_subsets,
            include_background: exp.include_background,
        })
    }
}

#[derive(Deserialize)]
pub struct ScenesConfig {
    #[serde(rename = "foreground-dBFS")]
    pub foreground_dbfs: Vec<i32>,
    #[serde(rename = "background-dBFS")]
    pub background_dbfs: Vec<i32>,
    pub foreground: Vec<StimulusBaseName>,
    pub background: Vec<StimulusBaseName>,
}
