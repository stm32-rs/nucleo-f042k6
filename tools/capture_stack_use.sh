#!/bin/bash

dirname="stack_usage_"`date -Iminutes`

mkdir -p $dirname

for i in `find examples -name "*.rs"`; do
        name=$(echo $i | sed -e "s,examples/,,g" -e "s,\.rs,,g")
        echo "Processing stack usage for example $name"
        #cargo +nightly call-stack --example $name  main > $dirname"/"$name.dot
        cargo +nightly call-stack --example $name Reset > $dirname"/"$name.dot
        dot -Tsvg -Nfontname='Fira Code' -Nshape=box $dirname"/"$name.dot > $dirname"/"$name.svg
done

echo "Captured stack usage for all examples into $dirname/"
