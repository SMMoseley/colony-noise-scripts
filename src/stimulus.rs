use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

use StimulusName::*;
#[derive(PartialEq, Hash, Eq, Clone, Debug)]
pub enum StimulusName {
    ForegroundBackground(StimulusBaseName, i32, StimulusBaseName, i32),
    Foreground(StimulusBaseName, i32),
}

impl StimulusName {
    pub fn foreground(&self) -> &StimulusBaseName {
        match self {
            ForegroundBackground(s, _, _, _) => s,
            Foreground(s, _) => s,
        }
    }

    pub fn group(&self) -> Option<i32> {
        match self {
            ForegroundBackground(_, fg_db, _, bg_db) => Some(bg_db - fg_db),
            Foreground(_, _) => None,
        }
    }
}

impl From<(&StimulusBaseName, i32)> for StimulusName {
    fn from((fg, fg_db): (&StimulusBaseName, i32)) -> Self {
        Foreground(fg.clone(), fg_db)
    }
}

impl From<((&StimulusBaseName, i32), (&StimulusBaseName, i32))> for StimulusName {
    fn from(
        ((fg, fg_db), (bg, bg_db)): ((&StimulusBaseName, i32), (&StimulusBaseName, i32)),
    ) -> Self {
        ForegroundBackground(fg.clone(), fg_db, bg.clone(), bg_db)
    }
}

impl Serialize for StimulusName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ForegroundBackground(fg, fg_db, bg, bg_db) => {
                serializer.serialize_str(&format!("{}{}_{}{}", fg, fg_db, bg, bg_db))
            }
            Foreground(fg, fg_db) => serializer.serialize_str(&format!("{}_{}", fg, fg_db.abs())),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Hash, Eq, Clone, Debug)]
pub struct StimulusBaseName(String);

impl StimulusBaseName {
    pub fn starts_with(&self, pat: &Self) -> bool {
        self.0.starts_with(&pat.0)
    }
}

impl fmt::Display for StimulusBaseName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct StimulusWithGroup {
    pub name: StimulusName,
    pub group: Option<i32>,
}
