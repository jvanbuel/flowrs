#!/bin/bash

set -e 

# Importing DAGs
MWAA_BUCKET=$(terraform output -raw mwaa_dag_bucket)
TMP=$(mktemp -d 2>/dev/null || mktemp -d -t 'tmpdir')

git clone --depth 1 --branch v2-10-stable "git@github.com:apache/airflow.git" "$TMP"
cd "$TMP/airflow/example_dags"

aws s3 sync --include '*.py' . "s3://$MWAA_BUCKET/dags/"
