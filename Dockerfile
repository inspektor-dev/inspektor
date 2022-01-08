FROM ubuntu:21.10

RUN apt-get update && apt-get install -y libssl-dev && apt-get install -y build-essential

COPY target/release/inspektor .

ENTRYPOINT  ["./inspektor"]