#!/bin/sh
sed -i "s|__API_BASE_URL__|${API_BASE_URL:-http://api.localhost}|g" /usr/share/nginx/html/index.html
exec nginx -g "daemon off;"
