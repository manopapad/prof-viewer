use reqwest::blocking::{RequestBuilder, Response};

use crate::http::fetch::ProfResponse;

pub fn fetch(request: RequestBuilder, on_done: Box<dyn FnOnce(Result<ProfResponse, String>) + Send>) {
    std::thread::Builder::new()
        .name("ehttp".to_owned())
        .spawn(move || {
            let text = request
                .send()
                .expect("test").text().expect("unable to get text");


            on_done(Ok(ProfResponse { body: text }))
        });
}
