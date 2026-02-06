//! This module contains the shared code between the client and the server for the auth example.

use bevy::reflect::Reflect;
use lightyear::netcode::PRIVATE_KEY_BYTES;
use serde::{Deserialize, Serialize};

// Define a internal port for the authentication backend
pub const AUTH_BACKEND_PORT: u16 = 4100;

/// A 32-byte array, used as a key for encrypting and decrypting packets and connect tokens.
pub type Key = [u8; PRIVATE_KEY_BYTES];

/// Response when calling the authentication endpoint.
#[derive(Reflect, Serialize, Deserialize, Clone)]
pub struct TokenResponse {
    pub token: Vec<u8>,
    pub client_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthPayload {
    pub client_id: u64,
    pub client_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewClientPayload {
    pub client_secret: String,
}
