FROM php:8.4-fpm-alpine

ADD https://github.com/simplesamlphp/simplesamlphp/releases/download/v2.5.0/simplesamlphp-2.5.0-full.tar.gz /tmp/simplesamlphp.tar.gz

RUN set -eux; \
    apk add --no-cache bash nginx openssl; \
    mkdir -p /opt; \
    tar -xzf /tmp/simplesamlphp.tar.gz -C /opt; \
    ln -s /opt/simplesamlphp-2.5.0 /opt/simplesamlphp; \
    rm /tmp/simplesamlphp.tar.gz; \
    mkdir -p /var/cache/simplesamlphp /run/nginx; \
    openssl req -newkey rsa:3072 -new -x509 -days 3652 -nodes \
    -subj "/CN=localhost" \
    -keyout /opt/simplesamlphp/cert/idp.pem \
    -out /opt/simplesamlphp/cert/idp.crt; \
    chown -R www-data:www-data /var/cache/simplesamlphp /opt/simplesamlphp/cert; \
    chmod 640 /opt/simplesamlphp/cert/idp.pem; \
    chmod 644 /opt/simplesamlphp/cert/idp.crt

COPY saml-test-server/acl.php /opt/simplesamlphp/config/
COPY saml-test-server/authsources.php /opt/simplesamlphp/config/
COPY saml-test-server/config.php /opt/simplesamlphp/config/
COPY saml-test-server/metadata/saml20-idp-hosted.php /opt/simplesamlphp/metadata/
COPY saml-test-server/metadata/saml20-sp-remote.php /opt/simplesamlphp/metadata/
COPY saml-test-server/nginx.conf /etc/nginx/nginx.conf
COPY saml-test-server/entrypoint.sh /usr/local/bin/start-saml-test-server
COPY publickey.cer /opt/simplesamlphp/cert/e-ks.crt

RUN chmod +x /usr/local/bin/start-saml-test-server

EXPOSE 8080

CMD ["/usr/local/bin/start-saml-test-server"]
