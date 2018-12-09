#!/bin/sh
find $1 -name grip_amxx_i386.so -print0 | xargs -0 -n1 ./check_static_linking_impl.sh 
