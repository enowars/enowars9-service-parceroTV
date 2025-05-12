#!/bin/sh
set -e
set -x

# Chown the mounted data volume
chown -R service:service "../data/"
ls -la
# Launch our service as user 'service'
exec su -s /bin/sh -c './target/release/backend' service