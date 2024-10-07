use serde::de::DeserializeOwned;

use crate::error::Error;

fn useragent() -> String {
    format!(
        "logix/{logix} ({os} {arch}) libcurl/{curl}",
        logix = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        curl = curl::Version::get().version()
    )
}

pub struct UrlFetch {
    url: String,
    easy: curl::easy::Easy,
}

impl UrlFetch {
    pub fn new(url: &str) -> Result<Self, Error> {
        let mut easy = curl::easy::Easy::new();
        easy.url(url)?;
        easy.useragent(&useragent())?;
        Ok(Self {
            url: url.into(),
            easy,
        })
    }

    pub fn get(mut self) -> Result<Response, Error> {
        let mut data = Vec::new();
        self.easy.get(true)?;
        {
            let mut transfer = self.easy.transfer();
            transfer.write_function(|buf| {
                data.extend_from_slice(buf);
                Ok(buf.len())
            })?;
            transfer.perform()?;
        }
        let status = self.easy.response_code()?;

        if !(200..=299).contains(&status) {
            return Err(Error::HttpRequest(
                self.url.clone(),
                format!("Server returned status {status}"),
            ));
        }

        Ok(Response {
            url: self.url,
            data,
        })
    }
}

pub struct Response {
    url: String,
    data: Vec<u8>,
}

impl Response {
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, Error> {
        serde_json::from_slice(&self.data)
            .map_err(|e| Error::HttpRequestJson(self.url.clone(), e.to_string()))
    }
}

impl From<curl::Error> for Error {
    fn from(e: curl::Error) -> Self {
        Self::CurlError(e)
    }
}
