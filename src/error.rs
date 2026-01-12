#[derive(Debug)]
pub enum Error
{
    OutOfBounds,
    InvalidHash,
}

impl std::fmt::Display for Error
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error{}
