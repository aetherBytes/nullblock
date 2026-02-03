use actix_web::web::Data;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;

// Benchmark data structures
#[derive(Debug, Clone)]
struct BenchmarkData {
    users: HashMap<String, json::Value>,
    wallets: HashMap<String, json::Value>,
    market_data: HashMap<String, json::Value>,
    defi_protocols: HashMap<String, json::Value>,
}

impl BenchmarkData {
    fn new() -> Self {
        let mut data = Self {
            users: HashMap::new(),
            wallets: HashMap::new(),
            market_data: HashMap::new(),
            defi_protocols: HashMap::new(),
        };

        // Initialize benchmark data
        data.initialize_benchmark_data();
        data
    }

    fn initialize_benchmark_data(&mut self) {
        // Create benchmark users
        for i in 0..100 {
            let user = json!({
                "id": format!("user-{}", i),
                "wallet_address": format!("0x{:040x}", i),
                "username": format!("user{}", i),
                "email": format!("user{}@nullblock.com", i),
                "created_at": "2024-01-01T00:00:00Z",
                "is_active": true
            });
            self.users.insert(format!("user-{}", i), user);
        }

        // Create benchmark wallets
        for i in 0..100 {
            let wallet = json!({
                "address": format!("0x{:040x}", i),
                "chain": "ethereum",
                "balance": 1000.0 + (i as f64 * 10.0),
                "tokens": vec![
                    json!({
                        "symbol": "ETH",
                        "name": "Ethereum",
                        "balance": 5.0 + (i as f64 * 0.1),
                        "price_usd": 3000.0
                    }),
                    json!({
                        "symbol": "USDC",
                        "name": "USD Coin",
                        "balance": 10000.0 + (i as f64 * 100.0),
                        "price_usd": 1.0
                    })
                ]
            });
            self.wallets.insert(format!("0x{:040x}", i), wallet);
        }

        // Create benchmark market data
        let symbols = vec![
            "BTC", "ETH", "SOL", "USDC", "USDT", "ADA", "DOT", "LINK", "UNI", "AAVE",
        ];
        for (i, symbol) in symbols.iter().enumerate() {
            let market_data = json!({
                "symbol": symbol,
                "price": 1000.0 + (i as f64 * 1000.0),
                "change_24h": -5.0 + (i as f64 * 2.0),
                "volume_24h": 1000000000.0 + (i as f64 * 1000000000.0),
                "market_cap": 10000000000.0 + (i as f64 * 10000000000.0),
                "timestamp": "2024-01-01T00:00:00Z"
            });
            self.market_data.insert(symbol.to_string(), market_data);
        }

        // Create benchmark DeFi protocols
        let protocols = vec![
            "uniswap",
            "aave",
            "compound",
            "curve",
            "sushiswap",
            "balancer",
            "yearn",
            "harvest",
        ];
        for (i, protocol) in protocols.iter().enumerate() {
            let defi_protocol = json!({
                "name": protocol,
                "tvl": 1000000000.0 + (i as f64 * 1000000000.0),
                "volume_24h": 10000000.0 + (i as f64 * 10000000.0),
                "apy": 5.0 + (i as f64 * 2.0),
                "risk_score": 0.1 + (i as f64 * 0.1)
            });
            self.defi_protocols
                .insert(protocol.to_string(), defi_protocol);
        }
    }
}

// Benchmark functions
fn benchmark_wallet_address_validation(c: &mut Criterion) {
    c.bench_function("wallet_address_validation", |b| {
        b.iter(|| {
            let address = black_box("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6");

            // Simulate validation logic
            let is_valid = address.starts_with("0x") && address.len() == 42;
            black_box(is_valid)
        })
    });
}

fn benchmark_auth_token_generation(c: &mut Criterion) {
    c.bench_function("auth_token_generation", |b| {
        b.iter(|| {
            let token = uuid::Uuid::new_v4().to_string();
            black_box(token)
        })
    });
}

fn benchmark_json_serialization(c: &mut Criterion) {
    c.bench_function("json_serialization", |b| {
        b.iter(|| {
            let data = json!({
                "success": true,
                "data": {
                    "symbol": "BTC",
                    "price": 45000.0,
                    "change_24h": 2.5,
                    "volume_24h": 25000000000.0,
                    "market_cap": 900000000000.0
                },
                "timestamp": "2024-01-01T00:00:00Z"
            });

            let serialized = serde_json::to_string(&data).unwrap();
            black_box(serialized)
        })
    });
}

fn benchmark_json_deserialization(c: &mut Criterion) {
    let json_string = r#"{
        "success": true,
        "data": {
            "symbol": "BTC",
            "price": 45000.0,
            "change_24h": 2.5,
            "volume_24h": 25000000000.0,
            "market_cap": 900000000000.0
        },
        "timestamp": "2024-01-01T00:00:00Z"
    }"#;

    c.bench_function("json_deserialization", |b| {
        b.iter(|| {
            let data: serde_json::Value = serde_json::from_str(black_box(json_string)).unwrap();
            black_box(data)
        })
    });
}

fn benchmark_hashmap_operations(c: &mut Criterion) {
    let mut data = BenchmarkData::new();

    c.bench_function("hashmap_lookup", |b| {
        b.iter(|| {
            let user = data.users.get("user-50").unwrap();
            black_box(user)
        })
    });

    c.bench_function("hashmap_insert", |b| {
        b.iter(|| {
            let key = format!("benchmark-{}", black_box(1000));
            let value = json!({
                "id": &key,
                "data": "benchmark_value"
            });
            data.users.insert(key, value);
        })
    });
}

fn benchmark_string_operations(c: &mut Criterion) {
    c.bench_function("string_parsing", |b| {
        b.iter(|| {
            let query = black_box("symbols=BTC,ETH,SOL&timeframe=24h&limit=100");
            let params: Vec<&str> = query.split('&').collect();
            let symbols: Vec<String> = params[0]
                .split('=')
                .nth(1)
                .unwrap()
                .split(',')
                .map(|s| s.to_string())
                .collect();
            black_box(symbols)
        })
    });

    c.bench_function("string_formatting", |b| {
        b.iter(|| {
            let symbol = black_box("BTC");
            let price = black_box(45000.0);
            let formatted = format!("{}: ${:.2}", symbol, price);
            black_box(formatted)
        })
    });
}

fn benchmark_vector_operations(c: &mut Criterion) {
    c.bench_function("vector_filter_map", |b| {
        b.iter(|| {
            let symbols = vec!["BTC", "ETH", "SOL", "USDC", "USDT"];
            let filtered: Vec<String> = symbols
                .iter()
                .filter(|s| s.len() == 3)
                .map(|s| s.to_string())
                .collect();
            black_box(filtered)
        })
    });

    c.bench_function("vector_sort", |b| {
        b.iter(|| {
            let mut prices = vec![45000.0, 3000.0, 100.0, 1.0, 1.0];
            prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
            black_box(prices)
        })
    });
}

fn benchmark_mutex_operations(c: &mut Criterion) {
    let data = Data::new(Mutex::new(BenchmarkData::new()));

    c.bench_function("mutex_lock_unlock", |b| {
        b.iter(|| {
            let _guard = data.lock().unwrap();
            // Simulate some work
            black_box(42)
        })
    });
}

fn benchmark_error_handling(c: &mut Criterion) {
    c.bench_function("result_handling", |b| {
        b.iter(|| {
            let result: Result<i32, String> = Ok(black_box(42));
            match result {
                Ok(value) => black_box(value),
                Err(_) => black_box(0),
            }
        })
    });

    c.bench_function("option_handling", |b| {
        b.iter(|| {
            let option: Option<i32> = Some(black_box(42));
            option.unwrap_or(black_box(0))
        })
    });
}

fn benchmark_crypto_operations(c: &mut Criterion) {
    c.bench_function("uuid_generation", |b| {
        b.iter(|| {
            let uuid = uuid::Uuid::new_v4();
            black_box(uuid)
        })
    });

    c.bench_function("hash_computation", |b| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            let data = black_box("test_data_for_hashing");
            data.hash(&mut hasher);
            black_box(hasher.finish())
        })
    });
}

// Benchmark groups
criterion_group!(
    basic_operations,
    benchmark_wallet_address_validation,
    benchmark_auth_token_generation,
    benchmark_string_operations,
    benchmark_vector_operations
);

criterion_group!(
    data_operations,
    benchmark_json_serialization,
    benchmark_json_deserialization,
    benchmark_hashmap_operations,
    benchmark_mutex_operations
);

criterion_group!(error_handling, benchmark_error_handling);

criterion_group!(crypto_operations, benchmark_crypto_operations);

criterion_main!(
    basic_operations,
    data_operations,
    error_handling,
    crypto_operations
);
