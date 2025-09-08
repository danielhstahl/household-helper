#!/bin/sh
envsubst '$BACKEND_SERVICE' < /etc/nginx/nginx.conf.template > /etc/nginx/nginx.conf
exec "$@"
