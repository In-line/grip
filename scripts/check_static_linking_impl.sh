#!/bin/sh
if ldd -r $1 | grep -E "SSL|BIO" ; then
  echo "ERROR: Library" $1 "has missing symbols and should be linked dynamically against OpenSSL. Static linking didn't work." >&2
  exit 1
fi
