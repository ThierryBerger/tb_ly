//! The client plugin.
//! The client will be responsible for:
//! - connecting to the server at Startup
//! - sending inputs to the server
//! - applying inputs to the locally predicted player (for prediction to work, inputs have to be applied to both the
//!   predicted entity and the server entity)
use shared::auth::{AuthPayload, NewClientPayload, TokenResponse};
use std::pin::pin;
use std::task::Poll;

use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, Task};
use lightyear::connection::client::ClientState;
use lightyear::netcode::ConnectToken;
use lightyear::prelude::client::*;
use lightyear::prelude::*;

use crate::{AuthPrefs, AuthTokenDef};

pub struct AuthClientPlugin {
    pub auth_backend_address: String,
}

impl Plugin for AuthClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConnectTokenRequestTask {
            auth_backend_addr: self.auth_backend_address.clone(),
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

        app.add_systems(Update, update_button_text);
        app.add_systems(
            Update,
            update_identity_button.run_if(resource_changed::<AuthPrefs>),
        );
        app.add_observer(on_update_status_message);
        app.add_observer(handle_connection);
        app.add_observer(handle_disconnection);
        app.add_observer(on_disconnect);
    }
}

#[derive(Component)]
struct StatusMessageMarker;

#[derive(Component)]
pub(crate) struct ClientButton;
#[derive(Component)]
pub(crate) struct IdentityButton;

/// Holds a handle to an io task that is requesting a `ConnectToken` from the backend
#[derive(Resource)]
struct ConnectTokenRequestTask {
    auth_backend_addr: String,
    task: Option<Task<Option<TokenResponse>>>,
}

/// If we have an io task that is waiting for a `ConnectToken`, we poll the task until completion,
/// then we retrieve the token and update the ClientConfig.
fn fetch_connect_token(
    mut connect_token_request: ResMut<ConnectTokenRequestTask>,
    client: Single<Entity, With<Client>>,
    commands: Commands,
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
                let current_auth = prefs.get_current_auth_mut();
                if let Some(token_response) = mb_token_response
                    && let Some(prefs) = current_auth
                {
                    info!("Using saved token.");
                    let client = client.into_inner();

                    start_lightyear_connect(commands, client, &token_response)?;
                    prefs.last_token = Some(token_response.token);
                    prefs.client_id = Some(token_response.client_id);
                    connect_token_request.task = None;
                    return Ok(());
                };
                info!("No current auth.");

                if let Some(current_auth) = current_auth {
                    current_auth.last_token = None;
                }
                connect_token_request.task = None;
                return Ok(());
            }
        }
    }
    Ok(())
}

fn start_lightyear_connect(
    mut commands: Commands,
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

/// Get a ConnectToken via a TCP connection to the authentication server
async fn create_client_from_auth_backend(
    auth_backend_address: &String,
    secret: String,
) -> Option<TokenResponse> {
    #[cfg(feature = "local")]
    let url = format!("{}/create_client", auth_backend_address);
    #[cfg(not(feature = "local"))]
    let url = format!("{}/create_client", auth_backend_address);
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
    auth_backend_address: &str,
    client_id: u64,
    secret: String,
) -> Option<TokenResponse> {
    #[cfg(feature = "local")]
    let url = format!("{}/connect_client", auth_backend_address);
    #[cfg(not(feature = "local"))]
    let url = format!("{}/connect_client", auth_backend_address);
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
    serde_json::from_slice::<TokenResponse>(&response.bytes).ok()
}

/// Remove all entities when the client disconnect
fn on_disconnect(
    _trigger: On<Insert, Disconnected>,
    mut commands: Commands,
    debug_text: Query<Entity, With<ClientIdText>>,
) {
    for entity in debug_text.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn update_identity_button(
    auth_pref: Res<AuthPrefs>,
    q: Query<&mut Text, With<IdentityButton>>,
) {
    for mut t in q {
        t.0 = format!("{:?}", auth_pref.current);
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
                    Text("Change identity".to_string()),
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
                    IdentityButton,
                ))
                .observe(
                    |_trigger: On<Pointer<Click>>, mut prefs: ResMut<AuthPrefs>| {
                        prefs.current = prefs.current.map_or(Some(1), |c| Some((c + 1) % 5));
                    },
                );
            // Connect Button:
            parent.spawn((
                Text("[Client]".to_string()),
                TextColor(Color::srgb(0.9, 0.9, 0.9).with_alpha(0.4)),
                TextFont::from_font_size(18.0),
                StatusMessageMarker,
                Node {
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ));
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
                    ClientButton,
                ))
                .observe(|_trigger: On<Pointer<Click>>, mut commands: Commands| {
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
            let auth_backend_addr = task_state.auth_backend_addr.clone();

            let task = if let Some(AuthTokenDef {
                secret,
                client_id: Some(client_id),
                ..
            }) = prefs.get_current_auth()
            {
                let secret = secret.clone();
                let client_id = *client_id;
                info!("Get a token for a already created client.");
                IoTaskPool::get().spawn_local(async move {
                    if let Some(response) = connect_existing_client_from_auth_backend(
                        &auth_backend_addr,
                        client_id,
                        // Use the same secret as before.
                        secret.clone(),
                    )
                    .await
                    {
                        return Some(response);
                    }
                    None
                    //create_client_from_auth_backend(&auth_backend_addr, secret).await
                })
            } else {
                info!("Create a new client and get its token.");
                // FIXME: use a random string
                let secret = "".to_string();
                prefs.current = Some(prefs.accounts.len());
                prefs.accounts.push(AuthTokenDef {
                    last_token: None,
                    client_id: None,
                    secret: secret.clone(),
                });
                IoTaskPool::get().spawn_local(async move {
                    create_client_from_auth_backend(&auth_backend_addr, secret).await
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

#[derive(Event, Debug)]
pub struct UpdateStatusMessage(pub String);

fn on_update_status_message(
    trigger: On<UpdateStatusMessage>,
    mut q: Query<&mut Text, With<StatusMessageMarker>>,
) {
    for mut text in &mut q {
        text.0 = trigger.event().0.clone();
    }
}
pub(crate) fn update_button_text(
    client: Single<&Client>,
    mut text_query: Query<&mut Text, (With<Button>, With<ClientButton>)>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        match client.state {
            ClientState::Disconnecting => {
                text.0 = "Disconnecting".to_string();
            }
            ClientState::Disconnected => {
                text.0 = "Connect".to_string();
            }
            ClientState::Connecting => {
                text.0 = "Connecting".to_string();
            }
            ClientState::Connected => {
                text.0 = "Disconnect".to_string();
            }
        }
    }
}

/// Component to identify the text displaying the client id

#[derive(Component)]
pub struct ClientIdText;

/// Listen for events to know when the client is connected, and spawn a text entity
/// to display the client id
#[expect(clippy::type_complexity)]
pub(crate) fn handle_connection(
    trigger: On<Add, Connected>,
    query: Query<&LocalId, Or<((With<LinkOf>, With<Client>), Without<LinkOf>)>>,
    mut commands: Commands,
) {
    if let Ok(client_id) = query.get(trigger.entity) {
        commands.spawn((
            Text(format!("Client {}", client_id.0)),
            TextFont::from_font_size(30.0),
            ClientIdText,
        ));
        commands.trigger(UpdateStatusMessage("Connected".to_string()));
    }
}

/// Listen for events to know when the client is disconnected, and print out the reason
/// of the disconnection
pub(crate) fn handle_disconnection(
    trigger: On<Add, Disconnected>,
    mut commands: Commands,
    debug_text: Query<Entity, With<ClientIdText>>,
    disconnected: Query<(Entity, &Disconnected)>,
) {
    commands.trigger(UpdateStatusMessage(format!(
        "Disconnected ({})",
        disconnected
            .get(trigger.entity)
            .map(|d| d.1.reason.as_ref())
            .unwrap_or(None)
            .unwrap_or(&"Unknown".to_string())
    )));
    for entity in debug_text.iter() {
        commands.entity(entity).despawn();
    }
}
