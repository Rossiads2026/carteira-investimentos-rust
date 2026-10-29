use crate::{
    security::{create_jwt, hash_password, validate_jwt, verify_password},
    views::{AssetDisplay, DashboardTemplate, LoginTemplate},
};
use askama::Template;
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AuthPayload {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct NewAssetPayload {
    pub symbol: String,
    pub asset_type: String,
    pub quantity: f64,
    pub average_price: f64,
}

fn render_login_page(error: Option<String>) -> Response {
    let template = LoginTemplate { error };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(error) => {
            tracing::error!("Falha ao renderizar a página de login: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn render_login() -> Response {
    render_login_page(None)
}

pub async fn login_user(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Form(payload): Form<AuthPayload>,
) -> impl IntoResponse {
    let user = match sqlx::query!(
        "SELECT id, password_hash FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return render_login_page(Some("E-mail ou senha incorretos.".into()));
        }
        Err(error) => {
            tracing::error!("Falha ao consultar usuário durante login: {error}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match verify_password(&payload.password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return render_login_page(Some("E-mail ou senha incorretos.".into()));
        }
        Err(error) => {
            tracing::error!("Falha ao verificar senha armazenada: {error}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    let token = match create_jwt(user.id) {
        Ok(token) => token,
        Err(error) => {
            tracing::error!("Falha ao gerar token de autenticação: {error}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let cookie = Cookie::build(("auth_token", token))
        .path("/")
        .http_only(true)
        .build();
    cookies.add(cookie);
    Redirect::to("/dashboard").into_response()
}

pub async fn render_dashboard(State(pool): State<PgPool>, cookies: Cookies) -> impl IntoResponse {
    let token_cookie = match cookies.get("auth_token") {
        Some(c) => c,
        None => return Redirect::to("/login").into_response(),
    };

    let claims = match validate_jwt(token_cookie.value()) {
        Ok(c) => c,
        Err(_) => return Redirect::to("/login").into_response(),
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Redirect::to("/login").into_response(),
    };

    let user = match sqlx::query!("SELECT email FROM users WHERE id = $1", user_id)
        .fetch_one(&pool)
        .await
    {
        Ok(u) => u,
        Err(sqlx::Error::RowNotFound) => return Redirect::to("/login").into_response(),
        Err(error) => {
            tracing::error!("Falha ao consultar usuário do painel: {error}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let db_assets = match sqlx::query!(
        "SELECT symbol, asset_type, quantity, average_price FROM assets WHERE user_id = $1",
        user_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(assets) => assets,
        Err(error) => {
            tracing::error!("Falha ao consultar ativos da carteira: {error}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let assets = db_assets
        .into_iter()
        .map(|row| {
            let total = &row.quantity * &row.average_price;

            AssetDisplay {
                symbol: row.symbol,
                asset_type: row.asset_type,
                quantity: row.quantity.to_string(),
                average_price: row.average_price.to_string(),
                total_invested: format!("{total:.2}"),
            }
        })
        .collect();

    let template = DashboardTemplate {
        email: user.email,
        assets,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(error) => {
            tracing::error!("Falha ao renderizar o painel: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn add_asset(
    State(pool): State<PgPool>,
    cookies: Cookies,
    Form(payload): Form<NewAssetPayload>,
) -> impl IntoResponse {
    let symbol = payload.symbol.trim();
    if symbol.is_empty()
        || symbol.len() > 20
        || payload.asset_type.trim().is_empty()
        || payload.asset_type.len() > 50
        || !payload.quantity.is_finite()
        || payload.quantity <= 0.0
        || !payload.average_price.is_finite()
        || payload.average_price <= 0.0
    {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let token_cookie = match cookies.get("auth_token") {
        Some(c) => c,
        None => return Redirect::to("/login").into_response(),
    };

    let claims = match validate_jwt(token_cookie.value()) {
        Ok(c) => c,
        Err(_) => return Redirect::to("/login").into_response(),
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Redirect::to("/login").into_response(),
    };

    let quantity = match bigdecimal::BigDecimal::try_from(payload.quantity) {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!("Quantidade de ativo inválida: {error}");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };
    let average_price = match bigdecimal::BigDecimal::try_from(payload.average_price) {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!("Preço médio inválido: {error}");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    if let Err(error) = sqlx::query!(
        "INSERT INTO assets (user_id, symbol, asset_type, quantity, average_price) VALUES ($1, $2, $3, $4, $5)",
        user_id,
        symbol.to_uppercase(),
        payload.asset_type.trim(),
        quantity,
        average_price
    )
    .execute(&pool)
    .await
    {
        tracing::error!("Falha ao cadastrar ativo: {error}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to("/dashboard").into_response()
}
#[derive(Deserialize)]
pub struct RegisterPayload {
    pub email: String,
    pub password: String,
}

pub async fn render_register() -> impl IntoResponse {
    Html("<h1>Cadastro</h1><form action='/register' method='POST'><input type='email' name='email' placeholder='E-mail' required><input type='password' name='password' placeholder='Senha' required><button type='submit'>Cadastrar</button></form>")
}

pub async fn register_user(
    State(pool): State<PgPool>,
    Form(payload): Form<RegisterPayload>,
) -> impl IntoResponse {
    let email = payload.email.trim();
    if email.is_empty()
        || email.len() > 255
        || !email.contains('@')
        || payload.password.is_empty()
        || payload.password.len() > 72
    {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let password_hash = match hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(error) => {
            tracing::error!("Falha ao gerar hash da senha para cadastro: {error}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match sqlx::query!(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2)",
        email,
        password_hash
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Redirect::to("/login").into_response(),
        Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
            StatusCode::CONFLICT.into_response()
        }
        Err(error) => {
            tracing::error!("Falha ao cadastrar usuário: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
