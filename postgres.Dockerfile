# This is installing the pgvector extension for postgres
FROM postgres:latest

RUN apt-get update && apt-get install -y \
    build-essential \
    sudo \
    git \
    postgresql-server-dev-all \
    && rm -rf /var/lib/apt/lists/*

RUN echo "en_US.UTF-8 UTF-8"> /etc/locale.gen 
RUN locale-gen

WORKDIR /tmp
RUN git clone --branch v0.6.0 https://github.com/pgvector/pgvector.git

WORKDIR /tmp/pgvector
RUN make
RUN make install