use axum::{
    extract::{Query, State},
    response::Response,
    Json,
};
use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use totp_rs::{Algorithm, Secret, TOTP};
use tower_sessions::Session;

use crate::{
    models::{app_state::AppState, user::User},
    print_green,
};

static ISSUER: &str = "ItsukiServer";
static USER_KEY: &str = "user";

/*****************************************/
/******** Request Parameters *********** */

// POST /auth/register
#[derive(Debug, Deserialize)]
pub struct RegisterSignInBodyParameter {
    pub email: String,
    pub password: String,
}

// GET /auth/otp/enable
#[derive(Debug, Deserialize)]
pub struct OTPResponseTypeQueryParameter {
    pub response_type: Option<OTPResponseType>,
}

// POST /auth/otp/verify
#[derive(Debug, Deserialize)]
pub struct VerifyOTPParameter {
    pub otp_token: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OTPResponseType {
    // secret key
    SecretKey,
    // This URL can be encoded as a QR code and scanned by authenticator apps
    Url,
    QrPng,
    QrBase64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SessionUserModel {
    pub email: String,
    pub signed_in: bool,
}

/*******************************/
/******** Handlers *********** */

// POST /auth/register
pub async fn register_handler(
    session: Session,
    State(state): State<AppState>,
    Json(params): Json<RegisterSignInBodyParameter>,
) -> Response {
    print_green!("POST /auth/register");
    println!("email: {}, password: {}", params.email, params.password);

    let mut current_users = state.db.lock().await;

    let existing: Vec<User> = current_users
        .clone()
        .into_iter()
        .filter(|u| u.email.to_lowercase() == params.email.to_lowercase())
        .collect();

    if !existing.is_empty() {
        return build_error_response(anyhow::anyhow!("User with email {} exists.", params.email));
    }

    let new = User {
        email: params.email.clone(),
        password: params.password,
        otp_secret: None,
        otp_verified: None,
    };

    current_users.push(new.clone());

    if let Err(error) = session
        .insert(
            USER_KEY,
            SessionUserModel {
                email: new.email.clone(),
                signed_in: true,
            },
        )
        .await
    {
        return build_error_response(anyhow::anyhow!("Error saving user into session: {}", error));
    };

    return build_json_response(&json!({
        "error": false,
        "user": new.to_response_value()
    }));
}

// POST /auth/signin
pub async fn signin_handler(
    session: Session,
    State(state): State<AppState>,
    Json(params): Json<RegisterSignInBodyParameter>,
) -> Response {
    print_green!("POST /auth/signin");
    println!("email: {}, password: {}", params.email, params.password);

    let mut current_users = state.db.lock().await;

    // println!("current user: {:?}", current_users);

    let Some(user) = current_users
        .iter_mut()
        .find(|user| user.email == params.email)
    else {
        return build_error_response(anyhow::anyhow!(
            "User with email {} does not exists.",
            params.email
        ));
    };

    if user.password != params.password {
        return build_error_response(anyhow::anyhow!("Invalid credential."));
    }

    let otp_verification_required = user.otp_verified == Some(true);

    if let Err(error) = session
        .insert(
            USER_KEY,
            SessionUserModel {
                email: params.email,
                signed_in: !otp_verification_required,
            },
        )
        .await
    {
        return build_error_response(anyhow::anyhow!("Error saving user into session: {}", error));
    };

    return build_json_response(&json!({
        "error": false,
        "otp_verification_required": otp_verification_required,
        "user": if otp_verification_required { Value::Null } else { user.to_response_value() }
    }));
}

// GET /auth/signout
pub async fn signout_handler(session: Session) -> Response {
    print_green!("GET /auth/signout");

    if let Err(error) = session.insert(USER_KEY, Value::Null).await {
        return build_error_response(anyhow::anyhow!("Error signing out user: {}", error));
    };

    return build_json_response(&json!({
        "error": false,
    }));
}

// GET /auth/otp/enable
pub async fn enable_otp_handler(
    session: Session,
    State(state): State<AppState>,
    Query(params): Query<OTPResponseTypeQueryParameter>,
) -> Response {
    print_green!("GET /auth/otp/enable");

    let mut current_users = state.db.lock().await;

    let session_user = match session.get::<SessionUserModel>(USER_KEY).await {
        Ok(u) => match u {
            Some(u) => u,
            None => {
                return build_error_response(anyhow::anyhow!(
                    "No user found for the current session."
                ))
            }
        },
        Err(error) => {
            return build_error_response(anyhow::anyhow!(error));
        }
    };

    let Some(user) = current_users
        .iter_mut()
        .find(|user| user.email == session_user.email)
    else {
        return build_error_response(anyhow::anyhow!(
            "User with email {} does not exists.",
            session_user.email
        ));
    };

    let otp = match generate_otp(&user, None) {
        Ok(otp) => otp,
        Err(error) => {
            return build_error_response(anyhow::anyhow!("Error generating otp: {}", error))
        }
    };

    let otp_secret: String = otp.get_secret_base32(); // equivalent to secret.to_encoded()

    user.otp_secret = Some(otp_secret);
    user.otp_verified = Some(false);

    return build_otp_response(otp, params.response_type);
}

// GET /auth/otp/disable
pub async fn disable_otp_handler(session: Session, State(state): State<AppState>) -> Response {
    print_green!("GET /auth/otp/disable");

    let mut current_users = state.db.lock().await;

    let session_user = match session.get::<SessionUserModel>(USER_KEY).await {
        Ok(u) => match u {
            Some(u) => u,
            None => {
                return build_error_response(anyhow::anyhow!(
                    "No user found for the current session."
                ))
            }
        },
        Err(error) => {
            return build_error_response(anyhow::anyhow!(error));
        }
    };
    if !session_user.signed_in {
        return build_error_response(anyhow::anyhow!("User has to sign in to disable 2FA."));
    }

    let Some(user) = current_users
        .iter_mut()
        .find(|user| user.email == session_user.email)
    else {
        return build_error_response(anyhow::anyhow!(
            "User with email {} does not exists.",
            session_user.email
        ));
    };

    user.otp_secret = None;
    user.otp_verified = None;

    return build_json_response(&json!({
        "error": false,
        "user": user.to_response_value()
    }));
}

// POST /auth/otp/verify
pub async fn verify_otp_handler(
    session: Session,
    State(state): State<AppState>,
    Json(params): Json<VerifyOTPParameter>,
) -> Response {
    print_green!("POST /auth/otp/verify");
    println!("Verifying token: {}", params.otp_token);

    let mut current_users = state.db.lock().await;

    let session_user = match session.get::<SessionUserModel>(USER_KEY).await {
        Ok(u) => match u {
            Some(u) => u,
            None => {
                return build_error_response(anyhow::anyhow!(
                    "No user found for the current session."
                ))
            }
        },
        Err(error) => {
            return build_error_response(anyhow::anyhow!(error));
        }
    };

    let Some(user) = current_users
        .iter_mut()
        .find(|user| user.email == session_user.email)
    else {
        return build_error_response(anyhow::anyhow!(
            "User with email {} does not exists.",
            session_user.email
        ));
    };

    let Some(saved_otp) = user.otp_secret.clone() else {
        return build_error_response(anyhow::anyhow!("User does not have otp enabled."));
    };

    let secret = Secret::Encoded(saved_otp);

    let otp = match generate_otp(&user, Some(secret)) {
        Ok(otp) => otp,
        Err(error) => {
            return build_error_response(anyhow::anyhow!("Error generating otp: {}", error))
        }
    };

    let is_valid = match otp.check_current(&params.otp_token) {
        Ok(b) => b,
        Err(error) => {
            return build_error_response(anyhow::anyhow!("Error validating otp: {}", error))
        }
    };

    // do not need to do anything if not valid
    // - if the user is already signed in, they are calling this handler for setting up otp and we should leave them as signed in
    // - if the user is not signed in, there is also nothing to update
    // - Also, we don't need to update otp_verified even when verification fails.
    if is_valid {
        user.otp_verified = Some(true);
        // sign in the user
        if let Err(error) = session
            .insert(
                USER_KEY,
                SessionUserModel {
                    email: user.email.clone(),
                    signed_in: true,
                },
            )
            .await
        {
            return build_error_response(anyhow::anyhow!("Error signning in user: {}", error));
        };
    }

    return build_json_response(&json!({
        "otp_verified": is_valid,
        "user": if !is_valid { Value::Null } else { user.to_response_value() }
    }));
}

/*****************************************/
/******** Helpers *********** */

fn generate_otp(user: &User, secret: Option<Secret>) -> anyhow::Result<TOTP> {
    let secret = secret.unwrap_or(Secret::generate_secret());

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes()?,
        Some(ISSUER.to_string()),
        user.email.to_owned(),
    )?;

    Ok(totp)
}

fn build_otp_response(otp: TOTP, response_type: Option<OTPResponseType>) -> Response {
    let response_type = response_type.unwrap_or(OTPResponseType::Url);
    match response_type {
        OTPResponseType::SecretKey => {
            return build_json_response(&json!({
                "otp_key": otp.get_secret_base32()
            }));
        }
        OTPResponseType::Url => {
            return build_json_response(&json!({
                "otp_auth_url": otp.get_url()
            }));
        }
        OTPResponseType::QrPng => {
            let Ok(bytes) = otp.get_qr_png() else {
                return build_error_response(anyhow::anyhow!("Error generating QR code."));
            };
            let mut bytes_header = HeaderMap::new();
            bytes_header.insert(CONTENT_TYPE, "image/png".parse().unwrap());
            return (bytes_header, bytes).into_response();
        }
        OTPResponseType::QrBase64 => {
            let Ok(base64) = otp.get_qr_base64() else {
                return build_error_response(anyhow::anyhow!("Error generating QR code."));
            };
            return build_json_response(&json!({
                "otp_qr_base64": base64
            }));
        }
    }
}

fn build_error_response(error: anyhow::Error) -> Response {
    let mut json_header = HeaderMap::new();
    json_header.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let mut response = Response::new(
        json!({
            "error": true,
            "message": format!("{}", error)
        })
        .to_string(),
    );
    *response.status_mut() = StatusCode::BAD_REQUEST;
    return (json_header, response).into_response();
}

fn build_json_response(body: &Value) -> Response {
    let mut json_header = HeaderMap::new();
    json_header.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    let response = Response::new(body.to_string());
    return (json_header, response).into_response();
}
