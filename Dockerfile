####################
# BACKEND BUILDER  #
####################

FROM docker.io/library/rust:slim-trixie AS backend-builder
WORKDIR /builder
RUN apt update && apt install build-essential curl wget file libssl-dev pkg-config -y
COPY ../backend/ .
RUN cargo build --all -r

FROM docker.io/library/debian:trixie-slim AS base
WORKDIR /app
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

FROM base AS backend
COPY --from=backend-builder /builder/target/release/oceaniam /app/
CMD [ "./oceaniam" ]

FROM base AS migration
COPY --from=backend-builder /builder/target/release/migration /app/
CMD [ "./migration" ]

####################
# FRONTEND BUILDER #
####################

FROM docker.io/library/node:24-alpine AS frontend-builder
WORKDIR /builder
COPY ../frontend/ .
RUN corepack enable pnpm && pnpm install
RUN pnpm build

FROM docker.io/ferronserver/ferron:2-alpine AS frontend
WORKDIR /var/www/ferron
COPY --from=frontend-builder /builder/dist/ .
CMD ["/usr/sbin/ferron" "--config-adapter" "docker-auto"]

####################
# DATABASE BUILDER #
####################

FROM docker.io/library/postgres:18 AS database
ADD https://github.com/citusdata/pg_cron.git#v1.6.7 /tmp/pg_cron
RUN apt-get update && \
		apt-mark hold locales && \
		apt-get install -y --no-install-recommends build-essential postgresql-server-dev-18 && \
		cd /tmp/pg_cron && \
		make clean && \
		make OPTFLAGS="" && \
		make install && \
		mkdir /usr/share/doc/pg_corn && \
		cp LICENSE README.md /usr/share/doc/pg_corn && \
		rm -r /tmp/pg_cron && \
		apt-get remove -y build-essential postgresql-server-dev-18 && \
		apt-get autoremove -y && \
		apt-mark unhold locales && \
		rm -rf /var/lib/apt/lists/*

