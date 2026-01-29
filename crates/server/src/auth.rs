//! The server side of the example.
//! It is possible (and recommended) to run the server in headless mode (without any rendering plugins).
//!
//! The server will:
//! - spawn a new player entity for each client that connects
//! - read inputs from the clients and move the player entities accordingly
//!
//! Lightyear will handle the replication of entities automatically if you add a `Replicate` component to them.
extern crate alloc;
use alloc::sync::Arc;
use async_compat::Compat;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use core::net::SocketAddr;
use lightyear::connection::client::Disconnecting;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::auth::{AuthPayload, Key, NewClientPayload, TokenResponse};
use std::sync::{LazyLock, RwLock};
use tower_http::cors::{Any, CorsLayer};

use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use lightyear::netcode::ConnectToken;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::settings::SHARED_SETTINGS;

pub static PRIVATE_KEY: LazyLock<Key> = LazyLock::new(|| {
    std::fs::read("private.key")
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .unwrap_or_else(|| {
            error!("Private key is null, be careful on prod!");
            [0u8; 32]
        })
});

pub struct AuthServerPlugin {
    pub game_server_addr: SocketAddr,
    pub auth_backend_addr: SocketAddr,
}

impl Plugin for AuthServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_disconnect_event);
        app.add_observer(handle_connect_event);

        let client_ids = Arc::new(RwLock::new(HashSet::default()));
        let client_secrets = Arc::new(RwLock::new(HashMap::default()));
        start_netcode_authentication_task(
            self.game_server_addr,
            self.auth_backend_addr,
            client_ids.clone(),
            client_secrets.clone(),
        );
        app.insert_resource(ClientIds(client_ids));
    }
}

/// This resource will track the list of Netcode client-ids currently in use, so that
/// we don't have multiple clients with the same id
/// it maps to a unique id to fetch into database.
#[derive(Resource, Default)]
struct ClientIds(Arc<RwLock<HashSet<u64>>>);

/// Update the list of connected client ids when a client disconnects
fn handle_disconnect_event(
    trigger: On<Add, Disconnected>,
    query: Query<&RemoteId, With<ClientOf>>,
    //client_ids: Res<ClientIds>,
) {
    let Ok(remote_id) = query.get(trigger.entity) else {
        return;
    };
    if let PeerId::Netcode(client_id) = remote_id.0 {
        info!(
            "Client disconnected: {}. Removing from ClientIds.",
            client_id
        );
        // TODO: track list of connected clients? I guess we can just use ecs :thinking:
        // We don't want to remove clients because their Ids should persist between sessions.
        //client_ids.0.write().unwrap().remove(&client_id);
    } else {
        // ?
    }
}

/// Update the list of connected client ids when a client connects
fn handle_connect_event(
    trigger: On<Add, Connected>,
    mut commands: Commands,
    query: Query<&RemoteId, With<ClientOf>>,
    client_ids: Res<ClientIds>,
) {
    let Ok(remote_id) = query.get(trigger.entity) else {
        return;
    };
    if let PeerId::Netcode(client_id) = remote_id.0 {
        info!("Client connected: {}. Adding to ClientIds.", client_id);
        client_ids.0.write().unwrap().insert(client_id);
    } else {
        info!(
            "Client connected but not authenticated! Disconnecting {}",
            remote_id
        );
        commands.entity(trigger.entity).insert(Disconnecting);
    }
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

async fn create_client(
    client_ids: axum::extract::Extension<Arc<RwLock<HashSet<u64>>>>,
    client_secrets: axum::extract::Extension<Arc<RwLock<HashMap<u64, String>>>>,
    game_server_addr: axum::extract::Extension<GameServerAddr>,
    Json(payload): Json<NewClientPayload>,
) -> Json<TokenResponse> {
    // generate a unique client_id
    let client_id = loop {
        let id = rand::rng().next_u64();
        let mut ids = client_ids.write().unwrap();
        if !ids.contains(&id) {
            ids.insert(id);
            break id;
        }
    };
    {
        client_secrets
            .write()
            .unwrap()
            .insert(client_id, payload.client_secret);
    }

    // generate netcode ConnectToken
    let token = ConnectToken::build(
        game_server_addr.0.0,
        SHARED_SETTINGS.protocol_id,
        client_id,
        *PRIVATE_KEY,
    )
    .generate()
    .expect("Failed to generate token");

    let token_bytes = token.try_into_bytes().expect("Failed to serialize token");

    Json(TokenResponse {
        token: token_bytes.to_vec(),
        client_id,
    })
}

async fn connect_client(
    client_secrets: axum::extract::Extension<Arc<RwLock<HashMap<u64, String>>>>,
    game_server_addr: axum::extract::Extension<GameServerAddr>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<TokenResponse>, AuthError> {
    // reject connection if client doesn't exist.
    let Ok(secrets) = client_secrets.read() else {
        return Err(AuthError::TokenCreation);
    };
    let Some(secret) = secrets.get(&payload.client_id).cloned() else {
        return Err(AuthError::TokenCreation);
    };
    drop(secrets);

    // FIXME: store hash,
    if secret != payload.client_secret {
        return Err(AuthError::WrongCredentials);
    }
    // generate netcode ConnectToken
    let token = ConnectToken::build(
        game_server_addr.0.0,
        SHARED_SETTINGS.protocol_id,
        payload.client_id,
        *PRIVATE_KEY,
    )
    .generate()
    .expect("Failed to generate token");

    let token_bytes = token.try_into_bytes().expect("Failed to serialize token");

    Ok(Json(TokenResponse {
        token: token_bytes.to_vec(),
        client_id: payload.client_id,
    }))
}

#[derive(Clone)]
pub struct GameServerAddr(pub SocketAddr);

/// Start a detached task that listens for incoming TCP connections and sends `ConnectToken`s to clients
fn start_netcode_authentication_task(
    game_server_addr: SocketAddr,
    auth_backend_addr: SocketAddr,
    client_ids: Arc<RwLock<HashSet<u64>>>,
    client_secrets: Arc<RwLock<HashMap<u64, String>>>,
) {
    IoTaskPool::get()
        .spawn(Compat::new(async move {
            let cors = CorsLayer::new()
                // allow `GET` and `POST` when accessing the resource
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                // allow requests from any origin
                .allow_origin(Any)
                .allow_headers(Any);
            let app = Router::new()
                .route("/create_client", post(create_client))
                .route("/connect_client", post(connect_client))
                .layer(cors)
                .layer(axum::extract::Extension(client_ids))
                .layer(axum::extract::Extension(client_secrets))
                .layer(axum::extract::Extension(GameServerAddr(game_server_addr)));

            println!("Auth server listening on http://{}", auth_backend_addr);
            let listener = tokio::net::TcpListener::bind(auth_backend_addr)
                .await
                .unwrap();
            let _ = axum::serve(listener, app).await;
        }))
        .detach();
}
