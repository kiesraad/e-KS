#!/bin/sh

set -eu

mkdir -p /var/cache/simplesamlphp /run/nginx
chown -R www-data:www-data /var/cache/simplesamlphp

php-fpm -D
exec nginx -g "daemon off;"
