#[derive(Debug)]
pub struct Error {
    // description: String,
    // source:
    // kind: Box<dyn std::error::Error>,
}

#[derive(Debug)]
pub enum ErrorKind {}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ErrorKind {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {
    // fn description(&self) -> &str {
    //     &self.inner
    // }

    // fn cause(&self) -> Option<&dyn std::error::Error> {}

    // fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {}
}
