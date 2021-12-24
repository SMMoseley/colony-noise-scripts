use super::Error;
use core::cmp::{Ordering, PartialOrd};
use dynfmt::{curly::SimpleCurlyFormat, Format};
use itertools::Itertools;
use serde::{Deserialize, Serialize, Serializer};
use std::{borrow::Borrow, collections::HashMap, convert::TryFrom, fmt, iter};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Stimulus<'a> {
    attributes: HashMap<AttributeLabel, StimulusAttribute>,
    format: &'a str,
    decisive_attribute: &'a AttributeLabel,
}

impl<'a> Stimulus<'a> {
    pub fn new<M>(attributes: M, format: &'a str, decisive_attribute: &'a AttributeLabel) -> Self
    where
        M: Into<HashMap<AttributeLabel, StimulusAttribute>>,
    {
        let attributes = attributes.into();
        Stimulus {
            attributes,
            format,
            decisive_attribute,
        }
    }

    pub fn decisive_attribute(&self) -> &StimulusAttribute {
        self.attributes
            .get(self.decisive_attribute)
            .expect("Stimulus does not contain decisive_attribute")
    }

    pub fn matches(&self, (label, attribute): &(&AttributeLabel, &StimulusAttribute)) -> bool {
        self.attributes
            .get(*label)
            .map(|a| {
                if label.inclusive_less_than {
                    a <= attribute
                } else {
                    a == *attribute
                }
            })
            .unwrap_or(false)
    }

    // shouldn't have to do this
    pub fn attributes(&self) -> HashMap<String, StimulusAttribute> {
        self.attributes
            .iter()
            .map(|(k, v)| {
                (
                    String::from(<AttributeLabel as Borrow<str>>::borrow(k)),
                    v.clone(),
                )
            })
            .collect()
    }
}

impl<'a> Serialize for Stimulus<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(
            &SimpleCurlyFormat
                .format(self.format, self.attributes())
                .unwrap_or_else(|_| panic!("could not format {:?}", self.attributes))
                .into_owned(),
        )
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
pub struct AttributeLabel {
    label: String,
    inclusive_less_than: bool,
}

impl Borrow<str> for AttributeLabel {
    fn borrow(&self) -> &str {
        &self.label
    }
}

impl From<&str> for AttributeLabel {
    fn from(label: &str) -> Self {
        AttributeLabel {
            label: label.into(),
            inclusive_less_than: false,
        }
    }
}

#[derive(Deserialize)]
struct PermissiveStimuliConfig {
    format: String,
    decisive_attribute: String,
    #[serde(flatten)]
    attributes: HashMap<String, AttributeConfig>,
}

#[derive(Deserialize)]
struct AttributeConfig {
    values: Vec<StimulusAttribute>,
    #[serde(default)]
    inclusive_less_than: bool,
}

#[derive(Deserialize)]
#[serde(try_from = "PermissiveStimuliConfig")]
pub struct StimuliConfig {
    format: String,
    decisive_attribute: AttributeLabel,
    values: HashMap<AttributeLabel, Vec<StimulusAttribute>>,
}

impl StimuliConfig {
    pub fn stimuli(&self) -> Vec<Stimulus<'_>> {
        self.values
            .iter()
            .map(|(label, values)| iter::repeat(label.clone()).zip(values.iter().cloned()))
            .multi_cartesian_product()
            .map(|attributes| {
                let attributes: HashMap<_, _> = attributes.into_iter().collect();
                let format = &self.format;
                let decisive_attribute = &self.decisive_attribute;
                Stimulus::new(attributes, format, decisive_attribute)
            })
            .collect()
    }

    pub fn attribute_labels(&self) -> impl Iterator<Item = &AttributeLabel> {
        self.values.keys()
    }

    pub fn list_variants(&self, label: &AttributeLabel) -> Option<Vec<&StimulusAttribute>> {
        self.values
            .get(label)
            .map(|attribute_config| attribute_config.iter().collect())
    }

    pub fn label_by_str<'a>(&'a self, label: &str) -> Option<&'a AttributeLabel> {
        Self::attribute_label_by_str(&self.values, label)
    }

    fn attribute_label_by_str<'a, T>(
        attributes: &'a HashMap<AttributeLabel, T>,
        label: &str,
    ) -> Option<&'a AttributeLabel> {
        attributes.keys().find(|l| l.label == label)
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
            .map(
                |(
                    label,
                    AttributeConfig {
                        inclusive_less_than,
                        values,
                    },
                )| {
                    let label = AttributeLabel {
                        label,
                        inclusive_less_than,
                    };
                    (label, values)
                },
            )
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
