#!/bin/sh
# Copyright 2025 SYNTON-DB Team
#
# Licensed under the Apache License, Version 2.0 (the "License");

set -e

echo "SYNTON-DB Docker Entrypoint"
echo "==========================="

# Print environment variables for debugging
if [ "${SYNTON_DEBUG:-false}" = "true" ]; then
    echo "Environment variables:"
    env | grep SYNTON_ || true
    echo ""
fi

# Default to server command if none specified
COMMAND="${1:-server}"

case "$COMMAND" in
    server)
        echo "Starting SYNTON-DB server..."
        echo "Config file: ${SYNTON_CONFIG_FILE:-/etc/synton-db/config.toml}"

        # Build command with optional config override
        if [ -n "${SYNTON_CONFIG_FILE}" ]; then
            exec synton-db-server --config "${SYNTON_CONFIG_FILE}"
        else
            exec synton-db-server
        fi
        ;;

    validate)
        echo "Validating configuration..."
        if [ -n "${SYNTON_CONFIG_FILE}" ]; then
            synton-db-server --config "${SYNTON_CONFIG_FILE}" --validate
        else
            synton-db-server --validate
        fi
        ;;

    shell)
        echo "Starting interactive shell..."
        exec /bin/sh
        ;;

    backup)
        echo "Creating backup..."
        BACKUP_DIR="${BACKUP_DIR:-/data/backups}"
        mkdir -p "$BACKUP_DIR"
        TIMESTAMP=$(date +%Y%m%d_%H%M%S)
        BACKUP_FILE="$BACKUP_DIR/synton-db-backup-$TIMESTAMP.tar.gz"
        tar -czf "$BACKUP_FILE" -C /data rocksdb
        echo "Backup created: $BACKUP_FILE"
        ls -lh "$BACKUP_FILE"
        ;;

    *)
        echo "Unknown command: $COMMAND"
        echo "Available commands:"
        echo "  server   - Start the SYNTON-DB server (default)"
        echo "  validate - Validate configuration"
        echo "  shell    - Start an interactive shell"
        echo "  backup   - Create a backup of the data directory"
        exit 1
        ;;
esac
