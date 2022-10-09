use std::borrow::Cow;

#[derive(Debug)]
pub enum ApiError {
    External(u16, Cow<'static, str>),
    Internal(Cow<'static, str>),
}

impl From<ureq::Error> for ApiError {
    fn from(err: ureq::Error) -> Self {
        match err {
            ureq::Error::Status(code, response) => Self::External(
                code,
                response
                    .into_string()
                    .map(Cow::from)
                    .unwrap_or_else(|_| Cow::from("")),
            ),
            ureq::Error::Transport(transport) => Self::Internal(
                transport
                    .message()
                    .map(|s| s.to_owned().into())
                    .unwrap_or_else(|| "Unknown Transport Error".into()),
            ),
        }
    }
}
