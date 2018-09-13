//! Helpers for the web-facing parts of the library.

mod routes;
mod static_files;

use rocket::{self, Rocket};

use self::static_files::StaticFiles;
use Config;

/// This function produces an instance of the [Rocket](Rocket) app that we are building.
pub fn app(_config: &Config) -> Rocket {
    rocket::ignite()
        .mount("/static", StaticFiles::default().into())
        .mount("/user", routes![routes::user::register])
        .mount("/", routes![routes::base::index])
}
