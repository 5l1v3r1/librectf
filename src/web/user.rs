use actix_web::{
    self,
    middleware::{Middleware, Response},
    App, HttpRequest, HttpResponse, Json,
};
use bcrypt;
use diesel::{self, prelude::*};
use jsonwebtoken::{self, Header};

use super::{errors::WebError, DbConn, State};
use models::{NewUser, User};

pub fn app(state: State) -> App<State> {
    App::with_state(state)
        .resource("/user/login", |r| r.post().with(login))
        .resource("/user/register", |r| r.post().with(register))
}

pub struct LoginMiddleware;

impl<S> Middleware<S> for LoginMiddleware {
    fn response(&self, _req: &HttpRequest<S>, resp: HttpResponse) -> actix_web::Result<Response> {
        Ok(Response::Done(resp))
    }
}

#[derive(Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginClaim {
    username: String,
    admin: bool,
}

fn login((req, form, db): (HttpRequest<State>, Json<LoginForm>, DbConn)) -> HttpResponse {
    use schema::users::dsl::*;
    let state = req.state();
    let form = form.into_inner();
    users
        .filter(email.eq(&form.email))
        .first::<User>(&*db)
        .map_err(|err| {
            error!("Diesel error on user/login: {}", err);
            HttpResponse::Unauthorized().json("Check your credentials.".to_owned())
        }).and_then(|user| {
            bcrypt::verify(&form.password, &user.password)
                .map(|_| user)
                .map_err(|_| {
                    HttpResponse::Unauthorized().json("Check your credentials.".to_owned())
                })
        }).map(|user| {
            let claim = LoginClaim {
                username: user.name.clone(),
                admin: user.admin,
            };

            // generate jwt
            // TODO don't expect() this
            let token =
                jsonwebtoken::encode(&Header::default(), &claim, state.get_secret_key().as_ref())
                    .expect("failed to generate jwt");
            // let cookie = Cookie::build("user", token)
            //     .same_site(SameSite::Strict)
            //     .finish();

            HttpResponse::Ok()
                // .cookie(cookie)
                .json(token)
        }).unwrap_or_else(|err| err)
}

#[derive(Deserialize)]
struct RegisterForm {
    email: String,
    name: String,
    password: String,
}

fn register((form, db): (Json<RegisterForm>, DbConn)) -> HttpResponse {
    use schema::users;
    let new_user: NewUser = match form.into_inner().into() {
        Ok(user) => user,
        Err(err) => {
            error!("Failed to register: {:?}", err);
            return HttpResponse::BadRequest().json(err.to_string());
        }
    };
    diesel::insert_into(users::table)
        .values(new_user)
        .execute(&*db)
        .map(|_| HttpResponse::Ok().json("Registration successful.".to_owned()))
        .unwrap_or_else(|err| {
            error!("Diesel error on user/register: {}", err);
            HttpResponse::BadRequest()
                .json("Failed to complete registration, contact an admin.".to_owned())
        })
}

// TODO: this should be a ValidationError instead of WebError
impl Into<Result<NewUser, WebError>> for RegisterForm {
    fn into(self) -> Result<NewUser, WebError> {
        Ok(NewUser {
            email: self.email,
            name: self.name,
            password: bcrypt::hash(&self.password, bcrypt::DEFAULT_COST)?,
        })
    }
}
