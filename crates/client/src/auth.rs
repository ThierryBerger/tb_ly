//! The client plugin.
//! The client will be responsible for:
//! - connecting to the server at Startup
//! - sending inputs to the server
//! - applying inputs to the locally predicted player (for prediction to work, inputs have to be applied to both the
//! predicted entity and the server entity)
use core::net::SocketAddr;
use serde_json::json;
use shared::auth::{AuthPayload, NewClientPayload, TokenResponse};
use std::pin::pin;
use std::task::Poll;

use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, Task};
use lightyear::connection::client::ClientState;
use lightyear::netcode::ConnectToken;
use lightyear::prelude::client::*;
use lightyear::prelude::*;

use crate::AuthPrefs;

pub struct AuthClientPlugin {
    pub auth_backend_address: SocketAddr,
}

impl Plugin for AuthClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConnectTokenRequestTask {
            auth_backend_addr: self.auth_backend_address,
            task: None,
        });

        // despawn the existing connect button from the Renderer if it exists
        // (because we want to replace it with one with specific behaviour)
        // This might need adjustment if the common renderer changes significantly
        if let Ok(button_entity) = app
            .world_mut()
            .query_filtered::<Entity, With<Button>>()
            .single(app.world())
        {
            app.world_mut().despawn(button_entity);
        }

        app.add_systems(Startup, spawn_connect_button);
        app.add_systems(Update, fetch_connect_token);
        app.add_observer(on_disconnect);
    }
}

/// Holds a handle to an io task that is requesting a `ConnectToken` from the backend
#[derive(Resource)]
struct ConnectTokenRequestTask {
    auth_backend_addr: SocketAddr,
    task: Option<Task<Option<TokenResponse>>>,
}

/// If we have an io task that is waiting for a `ConnectToken`, we poll the task until completion,
/// then we retrieve the token and update the ClientConfig.
fn fetch_connect_token(
    mut connect_token_request: ResMut<ConnectTokenRequestTask>,
    client: Single<Entity, With<Client>>,
    mut commands: Commands,
    mut prefs: ResMut<AuthPrefs>,
) -> Result {
    if let Some(task) = &mut connect_token_request.task {
        // Use try_recv or poll without blocking
        if task.is_finished() {
            pub fn now_or_never<F: Future>(future: F) -> Option<F::Output> {
                let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
                match pin!(future).poll(&mut cx) {
                    Poll::Ready(x) => Some(x),
                    _ => None,
                }
            }
            if let Some(mb_token_response) = now_or_never(task) {
                info!("Received ConnectToken, starting connection!");
                let Some(token_response) = mb_token_response else {
                    prefs.last_token = None;
                    prefs.secret = None;
                    connect_token_request.task = None;
                    return Ok(());
                };
                let client = client.into_inner();

                start_lightyear_connect(commands, client, &token_response)?;
                prefs.last_token = Some(token_response);
                connect_token_request.task = None;
            }
        }
    }
    Ok(())
}

fn start_lightyear_connect(
    mut commands: Commands<'_, '_>,
    client: Entity,
    token_response: &TokenResponse,
) -> Result<(), BevyError> {
    let connect_token = ConnectToken::try_from_bytes(&token_response.token)
        .expect("Failed to parse token from authentication server");
    commands.entity(client).insert(NetcodeClient::new(
        Authentication::Token(connect_token),
        NetcodeConfig::default(),
    )?);
    commands.trigger(Connect { entity: client });
    Ok(())
}

/// Component to identify the text displaying the client id
#[derive(Component)]
pub struct ClientIdText;

/// Get a ConnectToken via a TCP connection to the authentication server
async fn create_client_from_auth_backend(
    auth_backend_address: SocketAddr,
    secret: String,
) -> Option<TokenResponse> {
    let url = format!("http://{}/create_client", auth_backend_address);
    let payload = NewClientPayload {
        client_secret: secret,
    };
    let mut req = ehttp::Request::post(url, serde_json::to_vec(&payload).unwrap());
    req.headers
        .insert("Content-Type", "application/json; charset=utf8");
    req.headers.insert("Accept", "application/json");
    let response = ehttp::fetch_async(req).await.unwrap_or_else(|_| {
        panic!(
            "Failed to connect to authentication server on {:?}",
            auth_backend_address
        )
    });

    info!(
        "Received response: {:?}. Token len: {:?}",
        response,
        response.bytes.len()
    );

    serde_json::from_slice::<TokenResponse>(&response.bytes).ok()
}

async fn connect_existing_client_from_auth_backend(
    auth_backend_address: SocketAddr,
    client_id: u64,
    secret: String,
) -> Option<TokenResponse> {
    let url = format!("http://{}/connect_client", auth_backend_address);
    let payload = AuthPayload {
        client_id,
        client_secret: secret,
    };
    let mut req = ehttp::Request::post(url, serde_json::to_vec(&payload).unwrap());
    req.headers
        .insert("Content-Type", "application/json; charset=utf8");
    req.headers.insert("Accept", "application/json");
    let response = ehttp::fetch_async(req).await.ok().or_else(|| {
        error!(
            "Failed to connect to authentication server on {:?}",
            auth_backend_address
        );
        None
    })?;

    info!(
        "Received response: {:?}. Token len: {:?}",
        response,
        response.bytes.len()
    );
    let token_response = serde_json::from_slice::<TokenResponse>(&response.bytes).ok();
    token_response
}

/// Remove all entities when the client disconnect
fn on_disconnect(
    trigger: On<Insert, Disconnected>,
    mut commands: Commands,
    debug_text: Query<Entity, With<ClientIdText>>,
) {
    for entity in debug_text.iter() {
        commands.entity(entity).despawn();
    }
}

/// Create a button that allow you to connect/disconnect to a server
/// When pressing Connect, we will start an asynchronous request via TCP to get a ConnectToken
/// that can be used to connect
pub(crate) fn spawn_connect_button(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexEnd,
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Text("Connect".to_string()),
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    TextFont::from_font_size(20.0),
                    BorderColor::all(Color::BLACK),
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                ))
                .observe(|trigger: On<Pointer<Click>>, mut commands: Commands| {
                    commands.run_system_cached(connect_system);
                });
        });
}

fn connect_system(
    mut commands: Commands,
    mut task_state: ResMut<ConnectTokenRequestTask>,
    client: Single<(Entity, &Client)>,
    mut prefs: ResMut<AuthPrefs>,
) {
    let (client_entity, client) = client.into_inner();
    match client.state {
        ClientState::Disconnected => {
            // Check if we have a token saved, if we do, use it, otherwise create a new one.
            info!("Starting task to get ConnectToken");
            let auth_backend_addr = task_state.auth_backend_addr;

            let task = if let AuthPrefs {
                secret: Some(secret),
                last_token: Some(last_token),
            } = prefs.as_ref()
            {
                let secret = secret.clone();
                let last_token = last_token.clone();
                info!("Get a token for a already created client.");
                IoTaskPool::get().spawn_local(async move {
                    if let Some(response) = connect_existing_client_from_auth_backend(
                        auth_backend_addr,
                        last_token.client_id,
                        // Use the same secret as before.
                        secret.clone(),
                    )
                    .await
                    {
                        return Some(response);
                    }
                    create_client_from_auth_backend(auth_backend_addr, secret).await
                })
            } else {
                info!("Create a new client and get its token.");
                // FIXME: use a random string
                let secret = "".to_string();
                prefs.secret = Some(secret.clone());
                IoTaskPool::get().spawn_local(async move {
                    create_client_from_auth_backend(auth_backend_addr, secret).await
                })
            };
            task_state.task = Some(task);
        }
        _ => {
            info!("Disconnecting from server");
            commands.trigger(Disconnect {
                entity: client_entity,
            });
        }
    };
}
