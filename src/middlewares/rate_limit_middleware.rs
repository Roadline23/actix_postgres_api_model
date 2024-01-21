use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage, Result};
use std::{time::{Instant, Duration}, collections::HashMap};

pub struct RateLimitMiddleware {
    max_requests: usize,
    duration: Duration,
    requests: HashMap<String, (usize, Instant)>,
}

impl RateLimitMiddleware {
    pub fn new(max_requests: usize, duration: Duration) -> Self {
        RateLimitMiddleware {
            max_requests,
            duration,
            requests: HashMap::new(),
        }
    }
}
