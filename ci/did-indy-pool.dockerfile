FROM --platform=linux/amd64 ubuntu:20.04

ARG uid=1000
# ARG user=indy

RUN apt-get update -y && apt-get install -y \
    # common stuff
    git \
    wget \
    gnupg \
    apt-transport-https \
    ca-certificates \
    apt-utils

# ========================================================================================================
# Update repository signing keys
# --------------------------------------------------------------------------------------------------------
# Hyperledger
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 9692C00E657DDE61 && \
    # Sovrin
    apt-key adv --keyserver keyserver.ubuntu.com --recv-keys CE7709D068DB5E88
# ========================================================================================================

# Plenum
#  - https://github.com/hyperledger/indy-plenum/issues/1546
#  - Needed to pick up rocksdb=5.8.8
RUN echo "deb  https://hyperledger.jfrog.io/artifactory/indy focal dev"  >> /etc/apt/sources.list && \
    echo "deb http://security.ubuntu.com/ubuntu bionic-security main"  >> /etc/apt/sources.list && \
    echo "deb https://repo.sovrin.org/deb bionic master" >> /etc/apt/sources.list && \
    echo "deb https://repo.sovrin.org/sdk/deb bionic master" >> /etc/apt/sources.list

# Sovrin
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys CE7709D068DB5E88 

RUN apt-get update -y && apt-get install -y \
    # Python
    python3-pip \
    python3-nacl \
    # rocksdb python wrapper
    rocksdb=5.8.8 \
    libgflags-dev \
    libsnappy-dev \
    zlib1g-dev \
    libbz2-dev \
    liblz4-dev \
    libgflags-dev \
    # zstd is needed for caching in github actions pipeline
    zstd \
    # fpm
    ruby \
    ruby-dev \
    rubygems \
    gcc \
    make \
    # Indy Node and Plenum
    libssl1.0.0 \
    ursa=0.3.2-1 \
    # Indy SDK
    libindy=1.15.0~1625-bionic \
    # Need to move libursa.so to parent dir
    && mv /usr/lib/ursa/* /usr/lib && rm -rf /usr/lib/ursa

RUN pip3 install -U \
    # Required by setup.py
    setuptools==50.3.2 \
    # Still pinned. Needs to be update like in plenum
    'pip<10.0.0' \
    'pyzmq==18.1.0' \
	'supervisor~=4.2'


# install fpm
RUN gem install --no-document rake fpm

RUN apt-get -y autoremove \
    && rm -rf /var/lib/apt/lists/*

# Download, build and install indy-node. Will invalidate cache if new HEAD
ADD https://api.github.com/repos/domwoe/indy-node/git/refs/heads/feature/did-indy-new version.json
RUN git clone --single-branch -b feature/did-indy-new https://github.com/domwoe/indy-node.git && \
    pip install -e indy-node


RUN echo "[supervisord]\n\
logfile = /tmp/supervisord.log\n\
logfile_maxbytes = 50MB\n\
logfile_backups=10\n\
logLevel = error\n\
pidfile = /tmp/supervisord.pid\n\
nodaemon = true\n\
minfds = 1024\n\
minprocs = 200\n\
umask = 022\n\
user = indy\n\
identifier = supervisor\n\
directory = /tmp\n\
nocleanup = true\n\
childlogdir = /tmp\n\
strip_ansi = false\n\
\n\
[program:node1]\n\
command=start_indy_node Node1 0.0.0.0 9701 0.0.0.0 9702\n\
directory=/home/indy\n\
stdout_logfile=/tmp/node1.log\n\
stderr_logfile=/tmp/node1.log\n\
\n\
[program:node2]\n\
command=start_indy_node Node2 0.0.0.0 9703 0.0.0.0 9704\n\
directory=/home/indy\n\
stdout_logfile=/tmp/node2.log\n\
stderr_logfile=/tmp/node2.log\n\
\n\
[program:node3]\n\
command=start_indy_node Node3 0.0.0.0 9705 0.0.0.0 9706\n\
directory=/home/indy\n\
stdout_logfile=/tmp/node3.log\n\
stderr_logfile=/tmp/node3.log\n\
\n\
[program:node4]\n\
command=start_indy_node Node4 0.0.0.0 9707 0.0.0.0 9708\n\
directory=/home/indy\n\
stdout_logfile=/tmp/node4.log\n\
stderr_logfile=/tmp/node4.log\n"\
>> /etc/supervisord.conf

RUN useradd -ms /bin/bash -u $uid indy

RUN mkdir -p \
	/etc/indy \
	/var/lib/indy/backup \
	/var/lib/indy/plugins \
	/var/log/indy \
	&& chown -R indy /etc/indy /var/lib/indy /var/log/indy

USER indy

RUN echo "LEDGER_DIR = '/var/lib/indy'\n\
LOG_DIR = '/var/log/indy'\n\
KEYS_DIR = '/var/lib/indy'\n\
GENESIS_DIR = '/var/lib/indy'\n\
BACKUP_DIR = '/var/lib/indy/backup'\n\
PLUGINS_DIR = '/var/lib/indy/plugins'\n\
NODE_INFO_DIR = '/var/lib/indy'\n\
NETWORK_NAME = 'sandbox'\n\
LOG_LEVEL = 'DEBUG'\n\
logLevel = 'DEBUG'\n"\
>> /etc/indy/indy_config.py

ARG pool_ip=127.0.0.1

RUN generate_indy_pool_transactions --nodes 4 --clients 5 --nodeNum 1 2 3 4 --ips="$pool_ip,$pool_ip,$pool_ip,$pool_ip"

EXPOSE 9701 9702 9703 9704 9705 9706 9707 9708

CMD ["/usr/local/bin/supervisord"]
