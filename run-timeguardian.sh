#!/bin/bash
#
# TimeGuardian Helper Script
# This script helps run TimeGuardian with the necessary root privileges

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TIMEGUARDIAN_BIN="$SCRIPT_DIR/target/debug/TimeGuardian"

# Make sure the binary exists
if [ ! -f "$TIMEGUARDIAN_BIN" ]; then
  echo "Error: TimeGuardian binary not found at $TIMEGUARDIAN_BIN"
  echo "Please build the project first with 'cargo build'"
  exit 1
fi

# Check if we're already running as root
if [ "$(id -u)" -eq 0 ]; then
  # Already root, just run the application
  echo "Running TimeGuardian with root privileges..."
  "$TIMEGUARDIAN_BIN" "$@"
else
  echo "TimeGuardian needs root privileges to modify the hosts file."
  echo "Requesting sudo access..."
  
  # Request sudo and run the application
  sudo "$TIMEGUARDIAN_BIN" "$@"
fi