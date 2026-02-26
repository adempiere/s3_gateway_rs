FROM rust:1.93.1-bookworm as builder

LABEL maintainer="ysenih@erpya.com; EdwinBetanc0urt@outlook.com;" \
	description="API RESTful with rust to manage Resources/Files/Images with S3;"

ARG BUILD_VERSION="1.0.0"

WORKDIR /opt/apps/server

COPY . . /opt/apps/server/

RUN apt-get update && \
	apt-get install -y \
		pkg-config \
		libssl-dev && \
	rm -rf /var/lib/apt/lists/* && \
    cargo install --config net.git-fetch-with-cli=true --path . && \
    mv docker/.env /usr/local/cargo/bin/


FROM debian:bookworm-slim

# The argument should be duplicated at this stage
ARG BUILD_VERSION "1.0.0"

ENV \
    RUST_LOG="info" \
	PORT="7878" \
	ALLOWED_ORIGIN="*" \
    S3_URL="-" \
    BUCKET_NAME="-" \
    API_KEY="-" \
    SECRET_KEY="-" \
    MANAGE_HTTPS="N" \
    SSL_CERT_FILE="/opt/apps/server/cacert.pem" \
	TZ="America/Caracas" \
	VERSION=${BUILD_VERSION}

COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server

WORKDIR /opt/apps/server

COPY --from=builder /usr/local/cargo/bin/.env /opt/apps/server/.env

RUN apt-get update && \
    apt-get install -y pkg-config openssl libssl-dev tzdata && \
    rm -rf /var/lib/apt/lists/* && \
    sed -i "s|info|$RUST_LOG|g" /opt/apps/server/.env && \
	sed -i "s|7878|$PORT|g" /opt/apps/server/.env && \
    sed -i "s|allowed_origin|$ALLOWED_ORIGIN|g" /opt/apps/server/.env && \
    sed -i "s|s3_url|$S3_URL|g" /opt/apps/server/.env && \
    sed -i "s|bucket_name|$BUCKET_NAME|g" /opt/apps/server/.env && \
    sed -i "s|api_key|$API_KEY|g" /opt/apps/server/.env && \
    sed -i "s|secret_key|$SECRET_KEY|g" /opt/apps/server/.env && \
    sed -i "s|ssl_cert_file|$SSL_CERT_FILE|g" /opt/apps/server/.env && \
	sed -i "s|manage_https|$MANAGE_HTTPS|g" /opt/apps/server/.env && \
	sed -i "s|1.0.0-dev|$VERSION|g" /opt/apps/server/.env

RUN addgroup adempiere && \
	adduser --disabled-password --gecos "" --ingroup adempiere --no-create-home adempiere && \
	chown -R adempiere /opt/apps/server/ && \
	chmod +x /usr/local/bin/server && \
    echo "Set Timezone..." && \
	echo $TZ > /etc/timezone

USER adempiere

CMD ["server"]
