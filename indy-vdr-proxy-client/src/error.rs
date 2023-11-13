use std::error::Error;
use std::fmt;

pub enum VdrProxyClientError {
    HttpClientError(reqwest::Error),
    ParseError(url::ParseError),
    NonSuccessStatusCode(u16, String),
}

impl fmt::Display for VdrProxyClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VdrProxyClientError::HttpClientError(err) => write!(f, "HTTP request failed: {}", err),
            VdrProxyClientError::ParseError(err) => write!(f, "URL parsing failed: {}", err),
            VdrProxyClientError::NonSuccessStatusCode(_, msg) => {
                write!(f, "Non-success status code: {}", msg)
            }
        }
    }
}

impl fmt::Debug for VdrProxyClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VdrProxyClientError::HttpClientError(err) => {
                write!(f, "HTTP request failed: {:?}", err)
            }
            VdrProxyClientError::ParseError(err) => write!(f, "URL parsing failed: {:?}", err),
            VdrProxyClientError::NonSuccessStatusCode(code, body) => {
                write!(f, "Non-success status code: {} {}", code, body)
            }
        }
    }
}

impl Error for VdrProxyClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            VdrProxyClientError::HttpClientError(err) => Some(err),
            VdrProxyClientError::ParseError(err) => Some(err),
            VdrProxyClientError::NonSuccessStatusCode(_, _) => None,
        }
    }
}

impl From<reqwest::Error> for VdrProxyClientError {
    fn from(err: reqwest::Error) -> VdrProxyClientError {
        VdrProxyClientError::HttpClientError(err)
    }
}

impl From<url::ParseError> for VdrProxyClientError {
    fn from(err: url::ParseError) -> VdrProxyClientError {
        VdrProxyClientError::ParseError(err)
    }
}
