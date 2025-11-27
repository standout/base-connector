#!/bin/bash

# Test setup script for API Connector
# This script manages the WireMock mock server for testing

set -e

case "$1" in
  start)
    echo "Starting WireMock mock server..."
    docker compose -f docker-compose.test.yml up -d
    echo "Waiting for mock server to be ready..."

    # Wait for the server to be ready
    max_attempts=30
    attempts=0

    while [ $attempts -lt $max_attempts ]; do
      if curl -f http://localhost:8080/__admin/health > /dev/null 2>&1; then
        echo "Mock server is ready!"
        exit 0
      fi

      sleep 1
      attempts=$((attempts + 1))
    done

    echo "Warning: Mock server failed to start within 30 seconds"
    exit 1
    ;;

  stop)
    echo "Stopping WireMock mock server..."
    docker compose -f docker-compose.test.yml down
    ;;

  restart)
    $0 stop
    $0 start
    ;;

  status)
    if curl -f http://localhost:8080/__admin/health > /dev/null 2>&1; then
      echo "Mock server is running"
    else
      echo "Mock server is not running"
      exit 1
    fi
    ;;

  *)
    echo "Usage: $0 {start|stop|restart|status}"
    exit 1
    ;;
esac
