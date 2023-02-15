FROM ghcr.io/hyperledger/indy-node-container/indy_node:1.13.2-rc5-ubuntu20-main
RUN pip3 install "supervisor~=4.2"

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

RUN mkdir -p \
	/etc/indy \
	/var/lib/indy/backup \
	/var/lib/indy/plugins \
	/var/log/indy \
	&& chown -R indy:root /etc/indy /var/lib/indy /var/log/indy

USER indy

RUN echo "LEDGER_DIR = '/var/lib/indy'\n\
LOG_DIR = '/var/log/indy'\n\
KEYS_DIR = '/var/lib/indy'\n\
GENESIS_DIR = '/var/lib/indy'\n\
BACKUP_DIR = '/var/lib/indy/backup'\n\
PLUGINS_DIR = '/var/lib/indy/plugins'\n\
NODE_INFO_DIR = '/var/lib/indy'\n\
NETWORK_NAME = 'sandbox'\n"\
>> /etc/indy/indy_config.py

ARG pool_ip=127.0.0.1

RUN generate_indy_pool_transactions --nodes 4 --clients 5 --nodeNum 1 2 3 4 --ips="$pool_ip,$pool_ip,$pool_ip,$pool_ip"

EXPOSE 9701 9702 9703 9704 9705 9706 9707 9708

CMD ["/usr/local/bin/supervisord"]
