####################
# BACKEND BUILDER  #
####################

FROM docker.io/library/rust:slim-trixie AS backend-builder
WORKDIR /builder
RUN apt update && apt install build-essential curl wget file libssl-dev pkg-config -y
COPY backend/ .
RUN cargo build --all -r

FROM docker.io/library/debian:trixie-slim AS base
WORKDIR /app
COPY backend/config.toml .
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

FROM base AS backend
COPY --from=backend-builder /builder/target/release/oceaniam /app/
# Required runtime env: OCEANIAM_MASTER_KEY (64-char hex, 32 bytes)
# Generate with: openssl rand -hex 32
CMD [ "./oceaniam" ]

FROM base AS worker
COPY --from=backend-builder /builder/target/release/oceaniam-worker /app/
# Required runtime env: OCEANIAM_MASTER_KEY (64-char hex, 32 bytes)
# Generate with: openssl rand -hex 32
CMD [ "./oceaniam-worker" ]

FROM base AS migration
COPY --from=backend-builder /builder/target/release/migration /app/
# Required runtime env: OCEANIAM_MASTER_KEY (64-char hex, 32 bytes)
# Generate with: openssl rand -hex 32
CMD [ "./migration" ]

####################
# FRONTEND BUILDER #
####################

FROM docker.io/library/debian:trixie AS frontend-builder
WORKDIR /builder/frontend
RUN apt-get update && apt-get install -y \
    curl git ca-certificates unzip && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -m flutter

COPY frontend/.fvmrc frontend/pubspec.yaml frontend/pubspec.lock ./

COPY sdk/dart/ /builder/sdk/dart/
COPY frontend/ .

RUN chown -R flutter:flutter /builder

USER flutter
RUN curl -fsSL https://fvm.app/install.sh | bash
ENV PATH="/home/flutter/fvm/bin:$PATH"
RUN fvm flutter build web --release

FROM docker.io/library/nginx:1.29-alpine AS frontend
COPY docker/nginx/frontend.conf /etc/nginx/conf.d/default.conf
COPY --from=frontend-builder /builder/frontend/build/web/ /usr/share/nginx/html/

####################
#  GATEWAY BUILDER #
####################

FROM docker.io/library/nginx:1.29-alpine AS nginx
COPY docker/nginx/gateway.conf /etc/nginx/conf.d/default.conf

####################
# DATABASE BUILDER #
####################

FROM docker.io/library/postgres:18 AS database
COPY docker/postgres/initdb/ /docker-entrypoint-initdb.d/
RUN apt-get update && \
		apt-mark hold locales && \
		apt-get install -y --no-install-recommends \
				build-essential postgresql-server-dev-18 libreadline-dev zlib1g-dev flex bison libxml2-dev \
				libxslt-dev libssl-dev libxml2-utils xsltproc pkg-config libc++-dev \
				libc++abi-dev libglib2.0-dev libtinfo6 cmake libstdc++-12-dev \
				liblz4-dev libcurl4-openssl-dev ninja-build git ca-certificates libicu-dev libcurl4t64
RUN git clone https://github.com/duckdb/pg_duckdb.git /tmp/pg_duckdb && \
		cd /tmp/pg_duckdb && \
		git submodule update --init --recursive && \
		make clean && \
		make -j$(nproc) && \
		make install &&\
		mkdir /usr/share/doc/pg_duckdb && \
		cp LICENSE README.md /usr/share/doc/pg_duckdb && \
		rm -r /tmp/pg_duckdb
RUN apt-get remove -y build-essential postgresql-server-dev-18 libreadline-dev zlib1g-dev flex bison libxml2-dev \
				libxslt-dev libssl-dev libxml2-utils xsltproc pkg-config libc++-dev \
				libc++abi-dev libglib2.0-dev cmake libstdc++-12-dev \
				liblz4-dev ninja-build git libicu-dev  && \
		apt-get autoremove -y && \
		apt-mark unhold locales && \
		rm -rf /var/lib/apt/lists/*
