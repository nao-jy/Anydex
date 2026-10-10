use std::str::FromStr;

pub enum JobType {
    CollectDiff,
    ProcessFile,
}

pub struct UnknownJobType;

impl FromStr for JobType {
    type Err = UnknownJobType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CollectDiff" => Ok(Self::CollectDiff),
            "ProcessFile" => Ok(Self::ProcessFile),
            _ => Err(UnknownJobType),
        }
    }
}

impl ToString for JobType {
    fn to_string(&self) -> String {
        match self {
            Self::CollectDiff => "CollectDiff".into(),
            Self::ProcessFile => "ProcessFile".into(),
        }
    }
}
