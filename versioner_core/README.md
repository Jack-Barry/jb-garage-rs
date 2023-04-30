# `versioner_core`

For now this is being written as an executable, but in the future will just be
the lib for core functionality for versioner

## What is Versioner?

Versioner is my toy project to build something similar to [Semantic Release](https://semantic-release.gitbook.io/semantic-release/),
but written in Rust and with better support for monorepo style projects. I
like Semantic Release, but having worked with it in some monorepo style projects
with several different versionable items, it can be cumbersome.

## Goals for Versioner

- Similar philosphy for semantic versioning based on formatted commit messages.
  This should be configurable per repo
- Easy to config for monorepos. The basic concept is you have a `versioner.config.json`
  file, and can specify an array of `projects`. For each project, you tell it
  what dir the project lives in, and optionally give it a custom prefix for Git
  tags. You can also specify if that project should have prereleases (`beta`,
  `alpha`, etc.)
  - The tricky part with monorepos is determining what commits are relevant to a
    given project. This needs to be configurable per project. Some prebuilt plugins
    can be provided for things like:
    - Find all internal dependency changes affecting "x" project
    - Translate Serverless YAML references into dependency tree
    - Includes/ignores, etc.
- Language agnostic plugin system. Though plugins written in Rust will be the
  most reliable, I don't want that getting in the way from someone writing a plugin
  in their language of choice. An ancillary benefit of this is that the interface
  for plugins must be extra-well-defined in order to support such a plugin system
- Tests, tests, tests. I'm not off to a great start on this since I'm still getting
  used to Rust, but I'd like for this project to be thoroughly unit and integration
  tested
