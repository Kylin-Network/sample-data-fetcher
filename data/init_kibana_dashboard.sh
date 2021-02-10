#!/bin/bash

set -x

echo "Init kibana dashboard"
curl -XPOST -H "kbn-xsrf: true" \
    --form file="@kibana.ndjson" \
    ${KIBANA_HOST}/api/saved_objects/_import

