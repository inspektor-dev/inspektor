FROM ubuntu:21.10 AS builder

RUN apt-get update && apt-get install -y libssl-dev && apt-get install -y build-essential && apt-get install -y wget && apt-get install -y netcat && apt-get install -y ca-certificates

RUN  wget https://raw.githubusercontent.com/eficode/wait-for/v2.2.1/wait-for 

RUN chmod u+x ./wait-for

FROM builder

ARG CACHEBUST=1

COPY target/release/inspektor . 
