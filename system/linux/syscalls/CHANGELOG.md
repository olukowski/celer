# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/olukowski/celer/compare/celer_system_linux_syscalls-v0.1.0...celer_system_linux_syscalls-v0.1.1) - 2026-03-15

### Added

- *(syscalls)* support 32-bit x86 ([#6](https://github.com/olukowski/celer/pull/6))
- create very thin wrappers for syscalls

### Fixed

- *(syscalls)* replace usize with size_t

### Other

- run on aarch64 linux
- *(syscalls)* use strict provenance APIs
- *(coverage)* use coveralls and branch coverage
- *(miri)* return -ENOSYS for unsupported syscalls and skip
