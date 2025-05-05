use std::borrow::Cow;

#[derive(Debug)]
pub enum ApiError {
    External(u16, Cow<'static, str>),
    Internal(Cow<'static, str>),
}

impl From<ureq::Error> for ApiError {
    fn from(err: ureq::Error) -> Self {
        match err {
            ureq::Error::StatusCode(code) => Self::External(
                code,
                "".into()
            ),
            _ => Self::Internal("Unknown Error".into()),
        }
    }
}
