#!/bin/sh

# Generates a private key to sign webtransport netcode's connect tokens

openssl rand 32 > private.key
chmod 600 private.key