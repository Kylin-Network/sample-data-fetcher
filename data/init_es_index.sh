#!/bin/bash

curl -H 'Content-Type: application/json' \
    -XPUT --data "@es_index.json" \
    http://localhost:9200/kylin_access_tracking
