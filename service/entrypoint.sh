#!/bin/sh
set -e
set -x

sh db/start_db.sh
# Chown the mounted data volume
mkdir -p /service/data/videos /service/data/thumbnails /service/data/private /service/data/shorts /service/data/vtt
chown -R service:service "../data/"
chown -R service:service /service/data

python3 /service/cleanup.py &

# Launch our service as user 'service'
exec su -s /bin/sh -c './target/release/backend' service