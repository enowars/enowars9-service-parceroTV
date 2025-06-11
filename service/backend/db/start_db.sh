#!/usr/bin/env bash
set -e  # Exit on any errorgi
cd $(dirname "$0")
sqlite3 ../../data/parcerotv.db < parcerotv.sql