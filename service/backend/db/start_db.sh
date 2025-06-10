#!/usr/bin/env bash
set -e  # Exit on any error
cd $(dirname "$0")
sqlite3 ../../data/parcerotv.db < parcerotv.sql