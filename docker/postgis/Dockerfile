FROM postgis/postgis
RUN apt-get update && apt-get install -y osm2pgsql
COPY ./11_import_data.sh /docker-entrypoint-initdb.d/
