## TODO

- More badges wirh shields.io?
- Use GitHub code scanning?
- Socket type (UDP, raw) should be configured by parameter.
- What should happen if we receive an unexpected message (e.g., a duplicate)?
- Handle timeout-event in ping-data-buffer.
- Instant::now(): apply dependency inversion and mock it in tests.
- Cleanup PingError.
- TTL (needs IP packet to be sent)

## done

- Push coverage report to coveralls.
- Add clippy pedantic lints.
- Use scripts for CI whenever effective.
- Code coverage in CI. Upload coverage report to artefacts.
- Cargo clippy in CI.
- Check formatting in CI.
- Rename repo.
- Timeout is set when socket is created.
- Replace al println! by logger.
- Size of channels should be configured by parameter.
- Fix all warnings.
