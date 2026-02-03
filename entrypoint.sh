#!/bin/sh
set -eu

CERT_DIR="certificates/"
CERT_FILE="$CERT_DIR/cert.pem"
KEY_FILE="$CERT_DIR/key.pem"

mkdir -p "$CERT_DIR"

# Write secrets to disk
echo "$WT_CERT" > "$CERT_FILE"
echo "$WT_KEY"  > "$KEY_FILE"

# Lock permissions (important for TLS libs)
chmod 600 "$CERT_FILE" "$KEY_FILE"

echo "Certificates written to $CERT_DIR"

# Optional sanity check
openssl x509 -in "$CERT_FILE" -noout -subject || true

# Start Bevy app
exec /usr/local/bin/server