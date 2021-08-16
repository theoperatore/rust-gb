use actix_web::http::StatusCode;
use actix_web::{web, ResponseError};
use serde::Serialize;
use serde_json::{json, to_string_pretty};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Serialize, Debug)]
pub struct AppError {
  pub msg: String,
  pub status: u16,
}

impl Display for AppError {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    write!(f, "{}", to_string_pretty(self).unwrap())
  }
}

impl ResponseError for AppError {
  // builds the actual response to send back when an error occurs
  fn error_response(&self) -> web::HttpResponse {
    let err_json = json!({ "error": self.msg });
    web::HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json(err_json)
  }
}
