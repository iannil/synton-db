#!/bin/bash
# Copyright 2025 SYNTON-DB Team
#
# Restore script for SYNTON-DB data.
#
# Usage:
#   ./scripts/restore.sh <backup_file> [restore_path]
#
# Arguments:
#   backup_file - Path to the backup tar.gz file
#   restore_path - Where to restore the data (default: ./data/rocksdb)

set -e

# Check arguments
if [ $# -lt 1 ]; then
    echo "Usage: $0 <backup_file> [restore_path]"
    echo ""
    echo "Arguments:"
    echo "  backup_file  - Path to the backup tar.gz file"
    echo "  restore_path - Where to restore the data (default: ./data/rocksdb)"
    exit 1
fi

BACKUP_FILE="$1"
RESTORE_PATH="${2:-./data/rocksdb}"

echo "Starting SYNTON-DB restore..."
echo "Backup file: $BACKUP_FILE"
echo "Restore path: $RESTORE_PATH"

# Check if backup file exists
if [ ! -f "$BACKUP_FILE" ]; then
    echo "Error: Backup file does not exist: $BACKUP_FILE"
    exit 1
fi

# Verify backup integrity
echo "Verifying backup integrity..."
if ! tar -tzf "$BACKUP_FILE" > /dev/null 2>&1; then
    echo "Error: Backup file is corrupted or not a valid tar.gz archive!"
    exit 1
fi
echo "Backup verification passed!"

# Create restore directory if it doesn't exist
mkdir -p "$(dirname "$RESTORE_PATH")"

# Check if restore path already exists
if [ -d "$RESTORE_PATH" ]; then
    echo "Warning: Restore path already exists: $RESTORE_PATH"
    read -p "Do you want to overwrite? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Restore cancelled."
        exit 0
    fi

    # Create a backup of existing data before overwriting
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    BACKUP_BEFORE_OVERWRITE="./backups/before-restore-$TIMESTAMP.tar.gz"
    mkdir -p ./backups
    echo "Backing up existing data to: $BACKUP_BEFORE_OVERWRITE"
    tar -czf "$BACKUP_BEFORE_OVERWRITE" -C "$(dirname "$RESTORE_PATH")" "$(basename "$RESTORE_PATH")" 2>/dev/null || true

    # Remove existing data
    echo "Removing existing data..."
    rm -rf "$RESTORE_PATH"
fi

# Restore from backup
echo "Restoring from backup..."
tar -xzf "$BACKUP_FILE" -C "$(dirname "$RESTORE_PATH")"

# The extracted directory name might be different (original dirname from backup)
# Handle this case
EXTRACTED_DIR="$RESTORE_PATH"
if [ ! -d "$EXTRACTED_DIR" ]; then
    # Find what was actually extracted
    for dir in "$(dirname "$RESTORE_PATH")"/*/; do
        if [ -d "$dir" ]; then
            mv "$dir" "$RESTORE_PATH"
            break
        fi
    done
fi

# Verify restore
if [ -d "$RESTORE_PATH" ]; then
    echo "Restore completed successfully!"
    echo "Restored data to: $RESTORE_PATH"
else
    echo "Error: Restore failed - directory not found!"
    exit 1
fi

echo "Restore process completed!"
