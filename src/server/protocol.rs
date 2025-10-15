// This file defines the MCP protocol specifications, including message formats and communication methods.

pub const MCP_VERSION: &str = "1.0";

pub const REQUEST_GET_UTC_TIME: &str = "GET_UTC_TIME";
pub const RESPONSE_UTC_TIME: &str = "UTC_TIME_RESPONSE";

#[derive(Debug)]
pub struct UtcTimeRequest {
    pub request_type: String,
}

#[derive(Debug)]
pub struct UtcTimeResponse {
    pub response_type: String,
    pub utc_time: String,
}

impl UtcTimeRequest {
    pub fn new() -> Self {
        UtcTimeRequest {
            request_type: REQUEST_GET_UTC_TIME.to_string(),
        }
    }
}

impl UtcTimeResponse {
    pub fn new(utc_time: String) -> Self {
        UtcTimeResponse {
            response_type: RESPONSE_UTC_TIME.to_string(),
            utc_time,
        }
    }
}