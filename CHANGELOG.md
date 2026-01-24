# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/dotneB/duckdb-chess/releases/tag/v0.1.0) - 2026-01-24

### <!-- 0 -->⛰️ Features

- add parse error column
- handle invalid utf8
- optimize memory usage
- refactor lib modules
- add a function to export moves from a movetext
- add game deduplication
- migrate to redraiment duckdb extensions helpers
- use PNG Reader read_games
- use shakmaty to keep position
- use duckdb-slt
- add opening detection utilities

### <!-- 1 -->🐛 Bug Fixes

- fix makefile for linux
- fix makefile
- fix inside docker env var
- fix deploy script
- fix phonies, fix copy error

### <!-- 11 -->Other

- initial version of rust extension template
- add placeholders for testing
- add testing infrastructure
- add WIP support for python-based tester
- makefile improvements
- minor makefile fix
- switch to sqllogictest repo
- adding CI/CD
- attempt to fix CI
- add distribution matrix recipe
- correct behaviour for makefile recipe
- try out fix for linux workflow fix
- powering through ci hell
- add python3 extra toolchain
- attempt to fix windows
- use python for platform independent mkdir -p
- use python for platform independent cp
- make paths work on windows
- add known issue, static link c runtime on windows
- switch back to main
- switch to back duckdb repo
- install from github not local
- bogus commit to trigger ci
- incorrect file is copied
- try out like this
- venv should go first
- inline to ci debug, trying to get this thing through
- disable tests for linux_amd64_gcc4 for now
- platform autodetection as file, small refactor
- Merge remote-tracking branch 'samansmink/main'
- indentation problem
- switch to other ci tools repo for testing
- switcheroo
- small fixes, explain why linux_amd64 tests don't run
- pass correct matrix script
- add configure check to make life easier
- switch to workflow in ci tools
- forgot to remove cd
- remove deploy step: community extensions are the way
- Update readme
- Preparing pushing makefiles to ci tools
- restore borked set_duckdb_version target
- restore duped file for current ci
- now for real
- move makefiles out
- switch to duckdb/duckdb
- update readme
- add workaround for now
- Fix warning 'function ... should have a snake case name'
- Have package name be same as extension
- Pass EXTENSION_NAME to rust code
- Add example workaround to change create-type in Wasm
- Enable Wasm in workflow
- Add env version of Makefile
- Name example after crate
- Fix paths
- Add comments on crate-type switch
- Remove unnecessary code
- Skip threaded wasm for now
- bump ci tools
- remove duplicated configuration, bump ci tools
- bump duckdb to v1.2.0
- bump version in workflow too
- skip musl
- disable WASM for now
- bump to v1.2.1
- Merge pull request #16 from yutannihilation/enable-windows_amd64_mingw
- bump to v1.2.2
- bump MainDistributionPipeline.yml to v1.2.2
- Update DuckDB to v1.3.0
- More v1.3.0
- Invoke CI only on main branch
- Remove [main] condition from pull_request
- Merge pull request #23 from yutannihilation/ci/run-only-on-main
- bump to v1.4.0
- bump loadable-macros
- bum to v1.4.1
- Bump to v1.4.2
- Use main ci-tools both in-tree and in workflow
- bump to v1.4.3
- Bump cargo version

### <!-- 2 -->🚜 Refactor

- test suite
- filter movetext
- into module

### <!-- 6 -->🧪 Testing

- test target should test release

### <!-- 7 -->🤖 CI

- restore community extension-ci-tools

### <!-- 8 -->⚙️ Miscellaneous Tasks

- scaffold foundation specs
- clean up
- formatting
- cleanup
