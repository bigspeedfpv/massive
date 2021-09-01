#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;
pub mod models;

mod util;
use util::log;

pub const DEFAULT_PREFIX: &'static str = "d.";

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

use self::models::{Server, NewServer};

pub fn get_server(server_id: i64) -> Server {
    use schema::servers::dsl::*;
    use schema::servers;

    if server_id == 0 {
        return Server {
            id: 0,
            prefix: DEFAULT_PREFIX.to_string(),
            react: true,
        };
    }

    let connection = establish_connection();
    let server = servers.find(server_id)
        .first::<Server>(&connection);

    if let Ok(server) = server {
        server
    } else {
        let new_server = NewServer {
            id: server_id,
            prefix: DEFAULT_PREFIX,
            react: true,
        };

        let server = diesel::insert_into(servers::table)
            .values(&new_server)
            .get_result(&connection);

        match server {
            Ok(v) => {
                log::database(&format!("Server {} not found in db, added.", server_id));
                return v;
            },
            Err(why) => {
                log::error(&format!("Unable to add {} to databse: {:?}", server_id, why));
                panic!();
            }
        }
    }
}

pub fn update_server(server: Server) {
    use schema::servers::dsl::*;

    let connection = establish_connection();
    let server_to_update = diesel::update(servers.find(server.id))
        .set((prefix.eq(server.prefix), react.eq(server.react)))
        .get_result::<Server>(&connection)
        .expect(&format!("Unable to find server {}", server.id));
        
    log::database(&format!("Updated server: {:?}", server_to_update));
}