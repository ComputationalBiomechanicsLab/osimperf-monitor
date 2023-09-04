#!/bin/bash

echo "this goes to stdout"

echo "this goes to stderr" >&2

echo "again this goes to stdout"

echo "and again this goes to stderr" 1>&2
