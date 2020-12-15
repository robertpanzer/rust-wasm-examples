use std::time::Duration;

use log::error;
use log::info;
use log::warn;
use proxy_wasm::hostcalls::send_http_response;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_http_context(|_, _| -> Box<dyn HttpContext> {
        Box::new(HttpAuthRandom {
            auth_on_data: false,
            authenticated: false,
        })
    });
}

struct HttpAuthRandom {
    auth_on_data: bool,
    authenticated: bool,
}

impl HttpContext for HttpAuthRandom {
    fn on_http_request_headers(&mut self, _: usize) -> Action {
        let path = self.get_http_request_header(":path").unwrap();
        info!("Got new request to {}", path);

        match path.as_str() {
            "/onheader" => self.execute_auth_call(),
            "/ondata" => {
                self.auth_on_data = true;
                Action::Continue
            }
            _ => Action::Continue,
        }
    }

    fn on_http_request_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        info!("On data and I need to auth {}", self.auth_on_data);
        if !self.auth_on_data {
            Action::Continue
        } else if _end_of_stream {
            self.execute_auth_call()
        } else {
            Action::Pause
        }
    }

    fn on_http_response_headers(&mut self, _: usize) -> Action {
        info!("Sending response headers");
        self.set_http_response_header("Powered-By", Some("proxy-wasm"));
        self.set_http_response_header(
            "Authenticated",
            Some(if self.authenticated { "yes" } else { "no" }),
        );
        Action::Continue
    }
}

impl Context for HttpAuthRandom {
    fn on_http_call_response(&mut self, _token_id: u32, _: usize, _body_size: usize, _: usize) {
        info!("Got response for token {}", _token_id);

        for (key, value) in self.get_http_call_response_headers() {
            info!("Got response header {} -> {}", key, value);
        }
        self.authenticated = true;
        self.resume_http_request();
    }
}

impl HttpAuthRandom {
    fn execute_auth_call(&mut self) -> Action {
        let send_result = self.dispatch_http_call(
            "ext_web_service",
            vec![
                (":method", "GET"),
                (":path", "/sleep/1000"),
                (":authority", "sleeper.org"),
            ],
            None,
            vec![],
            Duration::from_secs(5),
        );

        match send_result {
            Ok(token) => {
                info!(
                    "Dispatching external request was successful, got token {}",
                    token
                );
                Action::Pause
            }
            Err(status) => {
                let msg = format!("Dispatching external request failed {:?}", status);
                warn!("Could not dispatch request: {}", msg);
                if let Err(err) = send_http_response(500, vec![], Some(msg.as_bytes())) {
                    error!("Could not send error response: {:?}", err)
                }
                Action::Continue
            }
        }
    }
}
