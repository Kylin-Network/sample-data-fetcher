#!/bin/bash

set -x

echo "Delete es index and recreate it"
curl -H 'Content-Type: application/json' 
    -XDELETE ${ES_HOST}/${ES_INDEX_NAME}
curl -H 'Content-Type: application/json' \
    -XPUT --data "@es_index.json" \
    ${ES_HOST}/${ES_INDEX_NAME}
