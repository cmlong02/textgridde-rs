use std::{
    fs::File,
    io::{BufReader, Read, Result},
    path::PathBuf,
};

use encoding_rs::Encoding;

pub enum Source {
    Path(PathBuf),
    String(String),
    StringVector(Vec<String>),
    Stream(Box<dyn Read>),
    File(File),
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

impl From<String> for Source {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<&str> for Source {
    fn from(str: &str) -> Self {
        Self::String(str.to_string())
    }
}

impl From<PathBuf> for Source {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
    }
}

impl From<File> for Source {
    fn from(file: File) -> Self {
        Self::File(file)
    }
}

impl From<Box<dyn Read>> for Source {
    fn from(stream: Box<dyn Read>) -> Self {
        Self::Stream(stream)
    }
}

/// Gets the content of a file or stream.
///
/// # Arguments
///
/// * `source` - One of the following:
///     * `Source::StringVector` - A vector of strings (what every other source is converted to).
///     * `Source::Stream` - A stream of text (what path/file is converted to).
///     * `Source::String` - A string that may be a path to a file or the content itself.
///     * `Source::Path` - A path to a file.
///     * `Source::File` - A file.
/// * `name` - The file name; only passed during recursion.
///
/// # Returns
///
/// A `Result` containing a tuple of a vector of strings and a string if successful, or an `std::io::Error` if parsing failed.
pub fn get_file_content(source: Source, name: Option<String>) -> Result<(Vec<String>, String)> {
    match source {
        Source::StringVector(string_vector) => {
            Ok((string_vector, name.unwrap_or("New TextGrid".to_string())))
        }
        Source::String(string) => {
            if PathBuf::from(&string).is_file() {
                return get_file_content(Source::Path(string.into()), None);
            }
            let content = string
                .split('\n')
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();
            let name = "New TextGrid".to_string();

            get_file_content(Source::StringVector(content), Some(name))
        }
        Source::Stream(mut stream) => {
            // Use encoding_rs to detect the BOM and convert to UTF-8 if necessary, since TextGrid files can sometimes be encoded in UTF-16.

            let mut buffer = Vec::new();
            stream.read_to_end(&mut buffer)?;

            let (encoding, _) = Encoding::for_bom(&buffer).unwrap_or((encoding_rs::UTF_8, 0));
            let content = encoding.decode(&buffer).0;

            let content = content
                .split('\n')
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();

            get_file_content(
                Source::StringVector(content),
                Some(name.unwrap_or("New TextGrid".to_string())),
            )
        }
        Source::Path(path) => {
            let file = File::open(path.clone())?;
            let name = path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("New Textgrid"))
                .to_str()
                .unwrap()
                .to_string();

            get_file_content(Source::File(file), Some(name))
        }
        Source::File(file) => {
            // Wrap the file in a BufReader to recurse with Source::Stream
            let reader = BufReader::new(file);
            let stream = Box::new(reader);

            get_file_content(
                Source::Stream(stream),
                Some(name.unwrap_or_else(|| "New TextGrid".to_string())),
            )
        }
    }
}

#[cfg(test)]
mod test_input {
    use std::io::Cursor;

    use crate::input;

    #[test]
    fn get_file_content() {
        let content = "xmin = 0\nxmax = 10";
        let source = input::Source::Stream(Box::new(Cursor::new(content)));
        let (content, name) = input::get_file_content(source, None).unwrap();
        let expected_content = vec!["xmin = 0".to_string(), "xmax = 10".to_string()];
        let expected_name = "New TextGrid".to_string();
        assert_eq!(content, expected_content);
        assert_eq!(name, expected_name);
    }

    #[test]
    fn utf16() {
        let content =
            input::get_file_content(input::Source::Path("./example/utf16.TextGrid".into()), None)
                .unwrap()
                .0;
        let expected_content =
            input::get_file_content(input::Source::Path("./example/long.TextGrid".into()), None)
                .unwrap()
                .0;

        assert_eq!(content, expected_content);
    }
}
