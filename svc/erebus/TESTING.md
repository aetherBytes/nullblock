# Erebus Testing Guide

This document describes the testing structure and conventions for the Erebus service.

## Testing Structure

The Erebus service follows standard Rust testing conventions:

```
svc/erebus/
├── src/                    # Source code
│   ├── main.rs            # Main application entry point
│   └── resources/         # Application modules
├── tests/                 # Integration tests
│   ├── integration_tests.rs    # End-to-end integration tests
│   └── unit_tests.rs           # Unit test helpers and utilities
├── benches/               # Performance benchmarks
│   └── performance_benchmarks.rs
└── Cargo.toml            # Dependencies and configuration
```

## Test Types

### 1. Unit Tests
- **Location**: `tests/unit_tests.rs`
- **Purpose**: Test individual components and functions in isolation
- **Scope**: Wallet validation, auth token generation, JSON parsing, etc.
- **Execution**: `cargo test`

### 2. Integration Tests
- **Location**: `tests/integration_tests.rs`
- **Purpose**: Test complete workflows and API endpoints
- **Scope**: Full authentication flow, market data retrieval, DeFi protocol integration
- **Execution**: `cargo test test_full_integration_pipeline`

### 3. Performance Benchmarks
- **Location**: `benches/performance_benchmarks.rs`
- **Purpose**: Measure performance of critical operations
- **Scope**: JSON serialization, hashmap operations, crypto operations
- **Execution**: `cargo bench`

## Running Tests

### All Tests
```bash
cargo test
```

### Specific Test Categories
```bash
# Unit tests only
cargo test --test unit_tests

# Integration tests only
cargo test --test integration_tests

# Specific integration test
cargo test test_full_integration_pipeline -- --nocapture

# Performance tests only
cargo test test_performance_benchmarks -- --nocapture

# Load simulation
cargo test test_load_simulation -- --nocapture
```

### Benchmarks
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark groups
cargo bench basic_operations
cargo bench data_operations
cargo bench error_handling
cargo bench crypto_operations
```

## Test Coverage

### Unit Tests Coverage
- **Wallet Operations**: Address validation, chain validation
- **Authentication**: Token generation, bearer token extraction
- **Market Data**: Symbol parsing, data validation
- **DeFi Protocols**: Protocol parsing, metrics validation
- **Analysis**: Request validation, confidence scoring
- **LLM**: Request validation, cost estimation
- **Health Checks**: Response structure validation

### Integration Tests Coverage
- **Health Check**: Server status verification
- **Authentication Flow**: Login, token generation, session management
- **Wallet Information**: Authenticated wallet data retrieval
- **Market Data**: Real-time price and volume data
- **DeFi Protocols**: Protocol metrics and analytics
- **Analysis Pipeline**: Request submission and result retrieval
- **LLM Generation**: AI model response generation
- **Error Handling**: Invalid tokens, missing auth, invalid requests

### Performance Benchmarks Coverage
- **Basic Operations**: String parsing, vector operations, UUID generation
- **Data Operations**: JSON serialization/deserialization, hashmap operations
- **Error Handling**: Result and Option handling patterns
- **Crypto Operations**: UUID generation, hash computation
- **Concurrency**: Mutex operations, concurrent access patterns

## Test Data

### Mock Data Structure
The integration tests use comprehensive mock data:

```rust
struct MockServiceState {
    users: HashMap<String, MockUser>,
    wallets: HashMap<String, MockWallet>,
    market_data: HashMap<String, MockMarketData>,
    defi_protocols: HashMap<String, MockDefiProtocol>,
    analysis_results: HashMap<String, MockAnalysisResult>,
    llm_responses: HashMap<String, MockLLMResponse>,
    auth_tokens: HashMap<String, String>,
}
```

### Test Scenarios
1. **Valid User Authentication**: Complete login flow with valid wallet address
2. **Invalid Authentication**: Rejection of invalid tokens and missing auth
3. **Market Data Retrieval**: Real-time price data for multiple symbols
4. **DeFi Protocol Analysis**: Protocol metrics and risk assessment
5. **Analysis Pipeline**: End-to-end analysis request and result retrieval
6. **LLM Generation**: AI-powered insights and recommendations
7. **Error Scenarios**: Graceful handling of various error conditions

## Performance Testing

### Benchmark Categories
1. **Basic Operations**: Core functionality performance
2. **Data Operations**: JSON and data structure performance
3. **Error Handling**: Error handling pattern performance
4. **Crypto Operations**: Cryptographic operation performance

### Load Testing
- **Concurrent Requests**: 100 concurrent requests simulation
- **Request Types**: Market data, DeFi protocols, authentication
- **Performance Metrics**: Response times, throughput, error rates

## Test Utilities

### Helper Functions
- `create_test_user()`: Generate test user data
- `create_test_wallet()`: Generate test wallet data
- `create_test_market_data()`: Generate test market data
- `assert_success_response()`: Verify successful HTTP responses
- `assert_client_error_response()`: Verify client error responses
- `assert_server_error_response()`: Verify server error responses

### Mock Service Handlers
- `mock_auth_login()`: Simulate authentication endpoint
- `mock_wallet_info()`: Simulate wallet information endpoint
- `mock_market_data()`: Simulate market data endpoint
- `mock_defi_protocols()`: Simulate DeFi protocols endpoint
- `mock_analysis_request()`: Simulate analysis request endpoint
- `mock_analysis_result()`: Simulate analysis result endpoint
- `mock_llm_generate()`: Simulate LLM generation endpoint
- `mock_health_check()`: Simulate health check endpoint

## Continuous Integration

### Test Commands for CI
```bash
# Run all tests with verbose output
cargo test --verbose

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html

# Run benchmarks to ensure performance
cargo bench --verbose

# Check for compilation errors
cargo check

# Run clippy for code quality
cargo clippy -- -D warnings

# Format code
cargo fmt -- --check
```

### Expected Test Results
- **Unit Tests**: All tests should pass
- **Integration Tests**: Complete end-to-end workflows should succeed
- **Performance Tests**: Response times should be within acceptable ranges
- **Load Tests**: System should handle concurrent requests without errors

## Debugging Tests

### Common Issues
1. **Test Timeouts**: Increase timeout values for slow operations
2. **Mock Data Issues**: Verify mock data initialization
3. **Async Test Issues**: Ensure proper async/await usage
4. **State Management**: Check for proper state cleanup between tests

### Debug Commands
```bash
# Run tests with debug output
RUST_LOG=debug cargo test -- --nocapture

# Run specific test with debug
RUST_LOG=debug cargo test test_full_integration_pipeline -- --nocapture

# Check test compilation
cargo test --no-run

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test
```

## Best Practices

### Test Organization
1. **Group Related Tests**: Use modules to organize related test functions
2. **Descriptive Names**: Use clear, descriptive test function names
3. **Setup and Teardown**: Properly initialize and clean up test data
4. **Mock Data**: Use realistic mock data that reflects production scenarios

### Test Reliability
1. **Deterministic Tests**: Tests should produce consistent results
2. **Isolation**: Tests should not depend on each other
3. **Clean State**: Each test should start with a clean state
4. **Error Handling**: Test both success and error scenarios

### Performance Considerations
1. **Benchmark Critical Paths**: Focus on performance-critical operations
2. **Realistic Data**: Use realistic data sizes and structures
3. **Multiple Scenarios**: Test various input sizes and conditions
4. **Regression Detection**: Monitor for performance regressions

## Future Enhancements

### Planned Improvements
1. **Property-Based Testing**: Add property-based tests using proptest
2. **Fuzzing**: Implement fuzzing for input validation
3. **Contract Testing**: Add contract tests for external service integration
4. **Load Testing**: Expand load testing scenarios
5. **Performance Monitoring**: Add performance regression detection

### Test Coverage Goals
- **Unit Test Coverage**: >90% line coverage
- **Integration Test Coverage**: All major workflows
- **Performance Test Coverage**: All critical operations
- **Error Scenario Coverage**: All error handling paths

