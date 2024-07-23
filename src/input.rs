use std::{fs::File, io::Read, path::PathBuf};

pub enum Source {
    Path(PathBuf),
    String(String),
    StringVector(Vec<String>),
    Stream(Box<dyn Read>),
    File(File),
}

impl From<PathBuf> for Source {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
    }
}

impl From<&str> for Source {
    fn from(str: &str) -> Self {
        Self::String(str.to_string())
    }
}

impl From<String> for Source {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<Vec<String>> for Source {
    fn from(string_vector: Vec<String>) -> Self {
        Self::StringVector(string_vector)
    }
}

impl From<Vec<&str>> for Source {
    fn from(string_vector: Vec<&str>) -> Self {
        Self::StringVector(
            string_vector
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect(),
        )
    }
}

impl From<Box<dyn Read>> for Source {
    fn from(stream: Box<dyn Read>) -> Self {
        Self::Stream(stream)
    }
}

impl From<File> for Source {
    fn from(file: File) -> Self {
        Self::File(file)
    }
}
