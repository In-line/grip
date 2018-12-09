#!/bin/sh
if readelf -a $1 | grep -i global | grep -i SSL ; then
  echo "ERROR: Library" $1 "is linked against OpenSSL." >&2
  exit 1
fi
