use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{self, FromRequest, Request};
use rocket::outcome::IntoOutcome;
use sha2::{Sha256, Digest};
use hex;

pub struct AuthenticatedUser;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| {
                if cookie.value() == "authenticated" {
                    Some(AuthenticatedUser)
                } else {
                    None
                }
            })
            .or_forward(Status::Unauthorized)
    }
}

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

pub fn set_auth_cookie(cookies: &CookieJar<'_>) {
    cookies.add_private(Cookie::new("user_id", "authenticated"));
}

pub fn remove_auth_cookie(cookies: &CookieJar<'_>) {
    cookies.remove_private(Cookie::new("user_id", "authenticated"));
}

