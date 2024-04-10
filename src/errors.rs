#[derive(Debug)]
enum MyError {
    #[allow(dead_code)]
    IoError(std::io::Error),
    #[allow(dead_code)]
    ParseError(std::num::ParseIntError),
}

impl From<std::io::Error> for MyError {
    fn from(e: std::io::Error) -> Self {
        MyError::IoError(e)
    }
}

impl From<std::num::ParseIntError> for MyError {
    fn from(e: std::num::ParseIntError) -> Self {
        MyError::ParseError(e)
    }
}