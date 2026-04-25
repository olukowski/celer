# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/olukowski/celer/compare/celer_system_linux_syscalls-v0.1.0...celer_system_linux_syscalls-v0.2.0) - 2026-04-25

### Added

- *(syscalls)* thin wrappers around raw syscalls ([#10](https://github.com/olukowski/celer/pull/10))
- *(syscalls)* add x86_64 support ([#9](https://github.com/olukowski/celer/pull/9))
- *(syscalls)* aarch64 support ([#8](https://github.com/olukowski/celer/pull/8))
- *(syscalls)* full coverage of Linux 1.0 syscalls ([#7](https://github.com/olukowski/celer/pull/7))
- *(syscalls)* support 32-bit x86 ([#6](https://github.com/olukowski/celer/pull/6))
- create very thin wrappers for syscalls

### Fixed

- repair syscalls rustdoc links
- *(syscalls)* replace usize with size_t

### Other

- run on aarch64 linux
- *(syscalls)* use strict provenance APIs
- *(coverage)* use coveralls and branch coverage
- *(miri)* return -ENOSYS for unsupported syscalls and skip
