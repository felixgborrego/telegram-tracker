# build tdlib
FROM debian:9 as tdlib-builder
RUN apt -y update
RUN apt install -y g++ ccache openssl cmake gperf make git libssl-dev libreadline-dev zlib1g zlib1g-dev
RUN openssl version
RUN uname -a
WORKDIR /
RUN  git clone --depth 1 --branch v1.6.0 https://github.com/tdlib/td.git /td
WORKDIR /td
RUN mkdir build
WORKDIR build
RUN cmake -DCMAKE_BUILD_TYPE=Release -DOPENSSL_ROOT_DIR=/usr/local/opt/openssl/ ..
RUN make -j 2
RUN cmake cmake --install .
# Generated at /td/build/libtdjson.so.1.6.0



FROM rust as telegram-tracker-builder
WORKDIR app
# Build dependencies
COPY Cargo.toml /app/Cargo.toml
COPY --from=tdlib-builder /td/build/libtd* /usr/local/lib/
RUN mkdir src \
    && echo "// dummy file" > src/lib.rs \
    && LD_LIBRARY_PATH=/usr/local/lib cargo build
# Build the app
COPY src src
RUN ls /usr/local/lib
RUN LD_LIBRARY_PATH=/usr/local/lib cargo build --release


FROM debian:9 as runtime
WORKDIR app
RUN apt -y update && apt install -y g++ ccache openssl && rm -rf /var/lib/apt/lists/*
ENV LD_LIBRARY_PATH="/app/:${LD_LIBRARY_PATH}"
COPY --from=telegram-tracker-builder /app/target/release/telegram-tracker /app
COPY --from=tdlib-builder /td/build/libtd* /app

ENTRYPOINT ["/app/telegram-tracker"]