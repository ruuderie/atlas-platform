#!/bin/bash
# ==============================================================================
# Script to synchronize local Grafana dashboard JSON files with the remote instance.
# ==============================================================================

GRAFANA_URL="https://grafana.oply.co"

echo "Please enter your Grafana API Token (Service Account Token):"
read -s TOKEN
echo ""

echo "Syncing dashboards to $GRAFANA_URL..."

# Enable nullglob so the loop won't run if no files are found
shopt -s nullglob

# Get the directory where the script is located to ensure paths are always relative to it
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

for file in "$SCRIPT_DIR/docs/grafana/"*.json; do
  echo "Uploading $file..."
  
  # Grafana's API requires the JSON payload to be structured as {"dashboard": { ... }, "overwrite": true}
  # The JSON files are already wrapped in a "dashboard" key, so we just add "overwrite: true"
  PAYLOAD=$(jq '. + {overwrite: true}' "$file")
  
  RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "$GRAFANA_URL/api/dashboards/db" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD")
  
  STATUS=$(echo "$RESPONSE" | grep "HTTP_STATUS" | cut -d':' -f2)
  BODY=$(echo "$RESPONSE" | sed '/HTTP_STATUS/d')
  
  if [ "$STATUS" = "200" ]; then
    echo "✅ Success!"
  else
    echo "❌ Failed to upload $file (Status: $STATUS)"
    echo "Response: $BODY"
  fi
done

echo "Done!"
