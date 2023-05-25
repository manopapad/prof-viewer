

#[cfg(target_arch = "wasm32")]
use reqwest::{Request, RequestBuilder, Response, ClientBuilder};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::blocking::{Request, RequestBuilder, Response, ClientBuilder, Client};



pub struct ProfResponse {
    // pub response: Response,
    pub body: String,
}


pub fn fetch(request: RequestBuilder, on_done: impl 'static + Send + FnOnce(Result<ProfResponse, String>)) {
    #[cfg(not(target_arch = "wasm32"))]
    crate::http::fetch_native::fetch(request, Box::new(on_done));

    #[cfg(target_arch = "wasm32")]
    crate::http::fetch_web::fetch(request, Box::new(on_done));
}