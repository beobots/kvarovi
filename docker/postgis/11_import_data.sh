#!/bin/bash
set -e

export PGUSER="$POSTGRES_USER"
export PGPASSWORD="$POSTGRES_PASSWORD"
export PGDATABASE="$POSTGRES_DB"

OSM_FILE=/input_data/serbia-latest.osm.pbf
if test -f "$OSM_FILE"; then
    osm2pgsql -d $POSTGRES_DB -U $POSTGRES_USER $OSM_FILE
fi