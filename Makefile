CMD ?= sam

build: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
build:
	${CMD} build --beta-features --debug

deploy: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
deploy:
	${CMD} deploy --beta-features --debug

list: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
list:
	${CMD} list resources

delete: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
delete:
	${CMD} delete --beta-features --debug

clean:
	cargo clean
	rm -rf ./.aws-sam

download-streets:
	target/debug/download_beo_streets | tee download.csv && sort download.csv | uniq | tr '[:upper:]' '[:lower:]' > beograd_streets/beograd_streets.csv

./docker/postgis/input_data/serbia-latest.osm.pbf:
	@wget -P ./docker/postgis/input_data https://download.geofabrik.de/europe/serbia-latest.osm.pbf

dev-bootstrap: ./docker/postgis/input_data/serbia-latest.osm.pbf
	@echo "bootstrap"

.PHONY: delete clean list deploy build download-streets