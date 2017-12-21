#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate toml;
extern crate jsonwebtoken as jwt;
extern crate md5;
extern crate argon2rs;
extern crate chrono;
extern crate lettre;
extern crate dotenv;
extern crate redis;
#[macro_use]
extern crate postgres;
extern crate r2d2;
extern crate r2d2_redis;
extern crate r2d2_postgres;
extern crate hex;
extern crate url;
extern crate uuid;

pub mod config;
pub mod common;
pub mod guards;
pub mod fairings;
pub mod storage;
pub mod models;
pub mod handlers;

use rocket::Rocket;
use rocket::fairing::AdHoc;
use rocket_contrib::Template;
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use r2d2_redis::RedisConnectionManager;
use storage::{Database, Cache};

pub fn create() -> Rocket {
    let config = config::parse();

    let postgres_manager = 
        PostgresConnectionManager::new(config.postgres.addr.as_str(), TlsMode::None)
            .expect("failed to initialize postgres connection manager");
    let database = Database::new(postgres_manager)
        .expect("failed to create database");

    let redis_manager = RedisConnectionManager::new(config.redis.addr.as_str())
        .expect("failed to initialize redis connection manager");
    let cache = Cache::new(redis_manager).expect("failed to create cache");

    rocket::ignite()
        .attach(AdHoc::on_response(fairings::ratelimit::on_response))
        .attach(Template::fairing())
        .mount(
            "/api/v1/",
            routes![
                   handlers::index,
                   handlers::role::query_roles,
                   handlers::user::signup,
                   handlers::user::signin,
                   handlers::contact::query_types,
                   handlers::contact::create_contact,
                   handlers::contact::select_contacts,
                   handlers::contact::verify_contact,
                   handlers::contact::remove_contact,
                   handlers::contact::send_verify_token,
                   handlers::profile::query_genders,
                   handlers::profile::create_profile,
                   handlers::profile::select_profile,
                   handlers::profile::remove_profile,
                   handlers::application::create_application,
                   handlers::application::select_applications,
                   handlers::application::select_application,
                   handlers::application::remove_application,
                   handlers::application::create_scope,
                   handlers::application::select_scopes,
                   handlers::application::remove_scope,
                   handlers::authorization::create_authorization,
                   handlers::authorization::select_authorizations,
                   handlers::authorization::remove_authorization,
                   handlers::authorization::preview_authorization,
                   handlers::ticket::create_ticket,
                   handlers::ticket::update_ticket,
               ],
        )
        .manage(config)
        .manage(database)
        .manage(cache)
}
