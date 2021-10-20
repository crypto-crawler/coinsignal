#!/bin/bash -e

if [[ -z "${INFLUXDB_URL}" ]]; then
  echo >&2 "ERROR: \$INFLUXDB_URL is not set"
  exit 1
fi

if [[ -z "${INFLUXDB_ORG}" ]]; then
  echo >&2 "ERROR: \$INFLUXDB_ORG is not set"
  exit 1
fi

if [[ -z "${INFLUXDB_BUCKET}" ]]; then
  echo >&2 "ERROR: \$INFLUXDB_BUCKET is not set"
  exit 1
fi

if [[ -z "${INFLUXDB_TOKEN}" ]]; then
  echo >&2 "ERROR: \$INFLUXDB_TOKEN is not set"
  exit 1
fi

sed -i 's|INFLUXDB_URL|'$INFLUXDB_URL'|g' /etc/grafana/provisioning/datasources/influxdb.yml
sed -i 's|INFLUXDB_ORG|'$INFLUXDB_ORG'|g' /etc/grafana/provisioning/datasources/influxdb.yml
sed -i 's|INFLUXDB_BUCKET|'$INFLUXDB_BUCKET'|g' /etc/grafana/provisioning/datasources/influxdb.yml
sed -i 's|INFLUXDB_TOKEN|'$INFLUXDB_TOKEN'|g' /etc/grafana/provisioning/datasources/influxdb.yml
