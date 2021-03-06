use core::{
    models::User,
    users::{LoginForm, RegisterForm},
    Error, UserErrorKind,
};
use http::uri::Uri;
use warp::Filter;
use wtforms::Form;

use crate::extractors::{db_conn, get, get_context, navbar, Context};
use crate::render::render_template;
use crate::session::Session;

pub fn get_login() -> Resp!() {
    warp::path::end()
        .and(navbar())
        .and(get_context())
        .and_then(|ctx: Context| render_template("users/login.html", ctx.into()))
}

pub fn post_login() -> Resp!() {
    warp::body::form()
        .and_then(|form: LoginForm| {
            form.validate()
                .map_err(|_| {
                    Error::user(
                        "bad username or password",
                        UserErrorKind::BadUsernameOrPassword,
                    )
                })
                .map_err(warp::reject::custom)
        })
        .and(db_conn())
        .and_then(|form, conn| core::users::login_user(&conn, &form).map_err(warp::reject::custom))
        .and(get::<Session>())
        .map(|user: User, mut session: Session| {
            session.set_user(user);
            warp::ext::set::<Session>(session);
            warp::redirect::redirect(Uri::from_static("/users/profile"))
        })
}

pub fn get_logout() -> Resp!() {
    warp::get2().map(|| {
        warp::ext::set::<Session>(Session::default());
        warp::redirect::redirect(Uri::from_static("/users/profile"))
    })
}

pub fn get_profile() -> Resp!() {
    warp::path::end()
        .and(navbar())
        .and(get_context())
        .and_then(|ctx: Context| render_template("users/profile.html", ctx.into()))
}

pub fn get_register() -> Resp!() {
    warp::path::end()
        .and(navbar())
        .and(get_context())
        .and_then(|ctx: Context| render_template("users/register.html", ctx.into()))
}

pub fn post_register() -> Resp!() {
    warp::body::form()
        .and_then(|form: RegisterForm| {
            form.validate()
                .map_err(|_| {
                    Error::user(
                        "bad registration info",
                        UserErrorKind::BadUsernameOrPassword,
                    )
                })
                .map_err(warp::reject::custom)
        })
        .and(db_conn())
        .and_then(|form, conn| {
            core::users::register_user(&conn, &form).map_err(warp::reject::custom)
        })
        .map(|_user_id: i32| warp::redirect::redirect(Uri::from_static("/users/login")))
}
