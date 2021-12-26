use super::Error;
use core::cmp::{Ordering, PartialOrd};
use dynfmt::{curly::SimpleCurlyFormat, Format};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap, convert::TryFrom, fmt, iter};

#[derive(Serialize, Clone, Debug)]
#[serde(into = "String")]
pub struct Stimulus<'a> {
    attributes: HashMap<AttributeLabel, StimulusAttribute>,
    config: &'a StimuliConfig,
}

impl<'a> Stimulus<'a> {
    pub fn new<M>(attributes: M, config: &'a StimuliConfig) -> Self
    where
        M: Into<HashMap<AttributeLabel, StimulusAttribute>>,
    {
        let attributes = attributes.into();
        Stimulus { attributes, config }
    }

    pub fn decisive_attribute(&self) -> &StimulusAttribute {
        self.attributes
            .get(&self.config.decisive_attribute)
            .expect("StimuliConfig does not contain decisive_attribute")
    }

    pub fn matches(&self, (label, attribute): &(&AttributeLabel, &StimulusAttribute)) -> bool {
        let inclusive_less_than = self.config.values.get(*label).unwrap().inclusive_less_than;
        self.attributes
            .get(*label)
            .map(|a| {
                if inclusive_less_than {
                    a <= attribute
                } else {
                    a == *attribute
                }
            })
            .unwrap_or(false)
    }
}

impl<'a> From<Stimulus<'a>> for String {
    fn from(stimulus: Stimulus) -> Self {
        SimpleCurlyFormat
            .format(&stimulus.config.format, &stimulus.attributes)
            .unwrap_or_else(|e| panic!("could not format {:?} {:?}", stimulus.attributes, e))
            .into_owned()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Hash, Eq, Clone, Debug)]
pub struct StimulusAttribute(AttributeKind);

impl From<&str> for StimulusAttribute {
    fn from(attribute: &str) -> Self {
        StimulusAttribute(Text(attribute.into()))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Hash, Eq, Clone, Debug)]
#[serde(untagged)]
enum AttributeKind {
    Numeric(i32),
    Text(String),
}

use AttributeKind::*;

impl PartialOrd<StimulusAttribute> for StimulusAttribute {
    fn partial_cmp(&self, other: &StimulusAttribute) -> Option<Ordering> {
        match (self, other) {
            (&StimulusAttribute(Numeric(lhs)), &StimulusAttribute(Numeric(rhs))) => {
                lhs.partial_cmp(&rhs)
            }
            _ => panic!("comparing a numeric attribute with a non-numeric attribute"),
        }
    }
}

impl fmt::Display for StimulusAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            AttributeKind::Numeric(num) => write!(f, "{}", num),
            AttributeKind::Text(string) => write!(f, "{}", string),
        }
    }
}

#[derive(Serialize, PartialEq, Hash, Eq, Clone, Debug)]
pub struct AttributeLabel(String);

impl Borrow<str> for AttributeLabel {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<&str> for AttributeLabel {
    fn from(label: &str) -> Self {
        AttributeLabel(label.into())
    }
}

#[derive(Deserialize)]
struct PermissiveStimuliConfig {
    format: String,
    decisive_attribute: String,
    #[serde(flatten)]
    attributes: HashMap<String, AttributeConfig>,
}

#[derive(Deserialize, Debug)]
struct AttributeConfig {
    values: Vec<StimulusAttribute>,
    #[serde(default)]
    inclusive_less_than: bool,
}

#[derive(Deserialize, Debug)]
#[serde(try_from = "PermissiveStimuliConfig")]
pub struct StimuliConfig {
    format: String,
    decisive_attribute: AttributeLabel,
    values: HashMap<AttributeLabel, AttributeConfig>,
}

impl StimuliConfig {
    pub fn stimuli(&self) -> Vec<Stimulus<'_>> {
        self.values
            .iter()
            .map(|(label, config)| iter::repeat(label.clone()).zip(config.values.iter().cloned()))
            .multi_cartesian_product()
            .map(|attributes| {
                let attributes: HashMap<_, _> = attributes.into_iter().collect();
                Stimulus::new(attributes, self)
            })
            .collect()
    }

    pub fn attribute_labels(&self) -> impl Iterator<Item = &AttributeLabel> {
        self.values.keys()
    }

    pub fn list_variants(&self, label: &AttributeLabel) -> Option<Vec<&StimulusAttribute>> {
        self.values
            .get(label)
            .map(|attribute_config| attribute_config.values.iter().collect())
    }

    pub fn label_by_str<'a>(&'a self, label: &str) -> Option<&'a AttributeLabel> {
        Self::attribute_label_by_str(&self.values, label)
    }

    fn attribute_label_by_str<'a, T>(
        attributes: &'a HashMap<AttributeLabel, T>,
        label: &str,
    ) -> Option<&'a AttributeLabel> {
        attributes.keys().find(|AttributeLabel(l)| l == label)
    }

    pub fn decisive_attribute(&self) -> &AttributeLabel {
        &self.decisive_attribute
    }
}

impl<'a> TryFrom<PermissiveStimuliConfig> for StimuliConfig {
    type Error = Error;

    fn try_from(config: PermissiveStimuliConfig) -> Result<Self, Self::Error> {
        let format = config.format;
        let values = config
            .attributes
            .into_iter()
            .map(|(label, values)| {
                let label = AttributeLabel(label);
                (label, values)
            })
            .collect();
        let decisive_attribute = Self::attribute_label_by_str(&values, &config.decisive_attribute)
            .ok_or(Error::DecisiveAttributeNotFound)?
            .clone();
        Ok(StimuliConfig {
            format,
            decisive_attribute,
            values,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_stimulus() {
        let attributes: HashMap<AttributeLabel, StimulusAttribute> =
            vec![("a", "hi"), ("b", "hello")]
                .into_iter()
                .map(|(k, v)| (AttributeLabel::from(k), StimulusAttribute::from(v)))
                .collect();
        let values: HashMap<AttributeLabel, AttributeConfig> = HashMap::new();
        let format = String::from("{a} {b}");
        let decisive_attribute = AttributeLabel::from("a");
        assert!(attributes.get("a").is_some());
        let config = StimuliConfig {
            format,
            decisive_attribute,
            values,
        };
        let stim = Stimulus::new(attributes, &config);
        assert_eq!(serde_json::to_string(&stim).unwrap(), "\"hi hello\"");
    }
}
