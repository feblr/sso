use std::collections::HashMap;
use std::convert::From;
use std::error::Error as StdError;
use std::fmt;
use std::io::{Cursor, Error as IoError};
use std::string::FromUtf8Error;

use rocket::http::{ContentType, Status as HttpStatus};
use rocket::response::{Responder, Response};
use rocket::Request;
use rocket_contrib::templates::Template;
use serde_json;

use super::models::Error as ModelError;
use hex::FromHexError;
use jwt::errors::Error as JwtError;
use postgres::error::UNIQUE_VIOLATION;
use r2d2::Error as R2d2Error;
use redis::RedisError;
use url::ParseError;

pub mod application;
pub mod authorization;
pub mod contact;
pub mod gender;
pub mod group;
pub mod invitation;
pub mod permission;
pub mod preference;
pub mod profile;
pub mod role;
pub mod summary;
pub mod ticket;
pub mod token;
pub mod totp;
pub mod user;

#[derive(Debug)]
pub enum Error {
    IO(IoError),
    Storage(R2d2Error),
    Redis(RedisError),
    Model(ModelError),
    Jwt(JwtError),
    Parse(ParseError),
    Utf8(FromUtf8Error),
    Params,
    Privilege,
    Forbidden,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IO(ref err) => err.fmt(f),
            Error::Storage(ref err) => err.fmt(f),
            Error::Redis(ref err) => err.fmt(f),
            Error::Model(ref err) => err.fmt(f),
            Error::Jwt(ref err) => err.fmt(f),
            Error::Parse(ref err) => err.fmt(f),
            Error::Utf8(ref err) => err.fmt(f),
            Error::Params => write!(f, "Params"),
            Error::Forbidden => write!(f, "Forbidden"),
            Error::Privilege => write!(f, "Privilege"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IO(ref err) => err.description(),
            Error::Storage(ref err) => err.description(),
            Error::Redis(ref err) => err.description(),
            Error::Model(ref err) => err.description(),
            Error::Jwt(ref err) => err.description(),
            Error::Parse(ref err) => err.description(),
            Error::Utf8(ref err) => err.description(),
            Error::Params => "Params",
            Error::Privilege => "Privilege",
            Error::Forbidden => "Forbidden",
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::IO(err)
    }
}

impl From<R2d2Error> for Error {
    fn from(err: R2d2Error) -> Error {
        Error::Storage(err)
    }
}

impl From<RedisError> for Error {
    fn from(err: RedisError) -> Error {
        Error::Redis(err)
    }
}

impl From<ModelError> for Error {
    fn from(err: ModelError) -> Error {
        Error::Model(err)
    }
}

impl From<JwtError> for Error {
    fn from(err: JwtError) -> Error {
        Error::Jwt(err)
    }
}

impl From<FromHexError> for Error {
    fn from(_err: FromHexError) -> Error {
        Error::Privilege
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        Error::Parse(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::Utf8(err)
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _req: &Request) -> Result<Response<'r>, HttpStatus> {
        println!("handler error: {}", self);
        let mut body = HashMap::new();

        let status = match self {
            Error::IO(ref _io_err) => {
                body.insert("errno", "50000000");
                body.insert("errmsg", "internal server io error");

                HttpStatus::InternalServerError
            }
            Error::Storage(ref _storage_err) => {
                body.insert("errno", "50000000");
                body.insert("errmsg", "internal server storage error");

                HttpStatus::InternalServerError
            }
            Error::Redis(ref _redis_err) => {
                body.insert("errno", "50000000");
                body.insert("errmsg", "internal server storage error");

                HttpStatus::InternalServerError
            }
            Error::Model(ref model_err) => match model_err {
                &ModelError::NotFound => {
                    body.insert("errno", "404000");
                    body.insert("errmsg", "internal server error");
                    HttpStatus::NotFound
                }
                &ModelError::QuotaLimit => {
                    body.insert("errno", "402000");
                    body.insert("errmsg", "reach quota limit, need payment");
                    HttpStatus::PaymentRequired
                }
                &ModelError::Database(ref pg_err) => match pg_err.code() {
                    Some(state) if *state == UNIQUE_VIOLATION => {
                        body.insert("errno", "40900001");
                        body.insert("errmsg", "internal server error");
                        HttpStatus::Conflict
                    }
                    _ => {
                        body.insert("errno", "50000002");
                        body.insert("errmsg", "internal server error");
                        HttpStatus::InternalServerError
                    }
                },
                &ModelError::InvalidParam(ref field, ref _err) => {
                    body.insert("errno", "40000000");
                    body.insert("errmsg", "invalid paramemter");
                    body.insert("field", field);
                    HttpStatus::BadRequest
                }
                _ => {
                    body.insert("errno", "50000002");
                    body.insert("errmsg", "internal server error");
                    HttpStatus::InternalServerError
                }
            },
            Error::Jwt(ref _jwt_err) => {
                body.insert("errno", "50000003");
                body.insert("errmsg", "internal server error");

                HttpStatus::Unauthorized
            }
            Error::Parse(ref _parse_error) => {
                body.insert("errno", "50000004");
                body.insert("errmsg", "internal server error");

                HttpStatus::InternalServerError
            }
            Error::Utf8(ref _utf8_error) => {
                body.insert("errno", "50000005");
                body.insert("errmsg", "internal server error");

                HttpStatus::InternalServerError
            }
            Error::Params => {
                body.insert("errno", "50000006");
                body.insert("errmsg", "internal server error");

                HttpStatus::BadRequest
            }
            Error::Forbidden => {
                body.insert("errno", "40300001");
                body.insert("errmsg", "action is forbidden");

                HttpStatus::Forbidden
            }
            Error::Privilege => {
                body.insert("errno", "50000008");
                body.insert("errmsg", "internal server error");

                HttpStatus::Unauthorized
            }
        };

        match serde_json::to_string(&body) {
            Ok(body) => {
                let response = Response::build()
                    .status(status)
                    .header(ContentType::JSON)
                    .sized_body(Cursor::new(body))
                    .finalize();
                Ok(response)
            }
            Err(_) => Err(HttpStatus::InternalServerError),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TemplateContext {
    title: String,
    welcome: String,
}

#[get("/")]
pub fn index() -> Template {
    Template::render(
        "index",
        &TemplateContext {
            title: String::from("Feblr SSO"),
            welcome: String::from("Welcome to Feblr SSO"),
        },
    )
}
