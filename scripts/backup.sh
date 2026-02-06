#!/bin/bash
# Copyright 2025 SYNTON-DB Team
#
# Backup script for SYNTON-DB data.
#
# Usage:
#   ./scripts/backup.sh [backup_path]
#
# Environment variables:
#   SYNTON_DATA_PATH - Path to RocksDB data directory (default: ./data/rocksdb)
#   BACKUP_PATH - Destination path for backups (default: ./backups)

set -e

# Default paths
SYNTON_DATA_PATH="${SYNTON_DATA_PATH:-./data/rocksdb}"
BACKUP_PATH="${BACKUP_PATH:-./backups}"
CUSTOM_PATH="${1:-}"

# Use custom path if provided
if [ -n "$CUSTOM_PATH" ]; then
    BACKUP_PATH="$CUSTOM_PATH"
fi

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_PATH"

# Generate timestamp
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="synton-db-backup-$TIMESTAMP"
BACKUP_FILE="$BACKUP_PATH/${BACKUP_NAME}.tar.gz"

echo "Starting SYNTON-DB backup..."
echo "Data path: $SYNTON_DATA_PATH"
echo "Backup file: $BACKUP_FILE"

# Check if data directory exists
if [ ! -d "$SYNTON_DATA_PATH" ]; then
    echo "Error: Data directory does not exist: $SYNTON_DATA_PATH"
    exit 1
fi

# Create backup
echo "Creating backup archive..."
tar -czf "$BACKUP_FILE" -C "$(dirname "$SYNTON_DATA_PATH")" "$(basename "$SYNTON_DATA_PATH")"

# Check if backup was successful
if [ $? -eq 0 ]; then
    SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
    echo "Backup completed successfully!"
    echo "Backup size: $SIZE"
    echo "Backup file: $BACKUP_FILE"
else
    echo "Error: Backup failed!"
    exit 1
fi

# Optional: Verify backup integrity
echo "Verifying backup integrity..."
if tar -tzf "$BACKUP_FILE" > /dev/null 2>&1; then
    echo "Backup verification passed!"
else
    echo "Warning: Backup verification failed!"
    exit 1
fi

# Optional: Clean old backups (keep last 7 days)
echo "Cleaning old backups (keeping last 7 days)..."
find "$BACKUP_PATH" -name "synton-db-backup-*.tar.gz" -mtime +7 -delete 2>/dev/null || true

echo "Backup process completed!"
