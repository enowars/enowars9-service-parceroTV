#!/bin/sh
set -e
set -x

# Chown the mounted data volume
chown -R service:service "../data/"
sh db/start_db.sh
# Launch our service as user 'service'
exec su -s /bin/sh -c './target/release/backend' service