# Contributing to tmux-gateway

Thanks for your interest in contributing! This guide will help you get started.

## Development Guidelines

- Make sure to work in DDD and functional core imperative shell style, i.e. the api network can be rest, grpc or graphql but the issue should be focused on the domain and not the api, so every task is api agnostic and relevant to all existing apis
- Prefer usage of existing libs rather creating custom logic and implementations, for example, if you need to parse a yaml file, use an existing yaml parsing library instead of writing your own parser
- Write tests for your changes to ensure they work as expected and to prevent regressions in the future
- Make sure files are small and focused on a single responsibility, if a file is getting too big, consider splitting it into smaller files
