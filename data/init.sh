#!/bin/bash

set -x

curl -H 'Content-Type: application/json' \
    -XPUT --data "@es_index.json" \
    ${ES_HOST}/${ES_INDEX_NAME}

curl -XPOST -H "kbn-xsrf: true" \
    --form file="@kibana.ndjson" \
    ${KIBANA_HOST}/api/saved_objects/_import?createNewCopies=true

