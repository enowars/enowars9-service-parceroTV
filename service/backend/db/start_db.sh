#!/usr/bin/env bash
cd $(dirname "$0")
sqlite3 ../../data/parcerotv.db < parcerotv.sql