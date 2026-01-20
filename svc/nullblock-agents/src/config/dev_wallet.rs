pub const DEV_WALLETS: &[&str] = &[
    "5wrmi85pTPmB4NDv7rUYncEMi1KqVo93bZn3XtXSbjYT",
];

pub fn is_dev_wallet(wallet: &str) -> bool {
    let is_match = DEV_WALLETS.contains(&wallet);
    if is_match {
        tracing::info!("🔥 DEV WALLET CONFIRMED: {}", wallet);
    }
    is_match
}

pub fn get_dev_wallet_models() -> Vec<&'static str> {
    vec![
        "anthropic/claude-sonnet-4",
        "anthropic/claude-3.5-sonnet",
        "openai/gpt-4-turbo",
        "meta-llama/llama-3.1-405b-instruct",
    ]
}

pub fn get_dev_preferred_model() -> &'static str {
    "anthropic/claude-sonnet-4"
}
