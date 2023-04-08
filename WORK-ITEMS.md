## OPEN WORK-ITEMS

- Add content to README.md.
- Size of payload should be configurable.
- Cargo fmt:
  - group_imports?
- Cargo audit.
- Fix allow(clippy::missing_errors_doc).
- Can we have coverage of the extern c code?
- Are timeouts handled corretly everywhere?
- Instant::now(): apply dependency inversion and mock it in tests.
- What should happen if we receive an unexpected message (e.g., a duplicate)?
- Should we test RawSocket::recv_from? Unit test? Can we test Raw socket also in an integration test elegantly?
- More badges wirh shields.io?
- How would the code look like if we would refactor towards more free functions instead of methods?

## done

- In ping_sender.rs and in ping_receiver.rs raise test coverage.
- The timeout time of the socket should be configurable.
- After adding TTL, reevaluate our tests/test coverage/design.
- The count in PingRunner does not seem to work. (Try the cli example.)
- TTL
- Cleanup PingError.
- Integrate type for sequence number.
- Socket type (UDP, raw) should be configured by parameter.
- Code coverage badge in readme.
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
