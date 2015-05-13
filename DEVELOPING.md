# Releasing

When building a tag, travis will automatically upload
a linux version of the CLI.

Sadly, cross-compilation with RUst is still difficult,
so the OS X version must be deployed manually by
running

        ./release.sh

on OS X, and then uploading the result.
