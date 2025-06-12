#!/bin/sh
set -e
set -x

sh db/start_db.sh
# Chown the mounted data volume
chown -R service:service "../data/"
chown -R service:service /service/data



# Launch our service as user 'service'
exec su -s /bin/sh -c './target/release/backend' service