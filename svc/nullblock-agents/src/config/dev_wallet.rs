pub const DEV_WALLETS: &[&str] = &[
    "5wrmi85pTPmB4NDv7rUYncEMi1KqVo93bZn3XtXSbjYT",
];

pub fn is_dev_wallet(wallet: &str) -> bool {
    DEV_WALLETS.contains(&wallet)
}

pub fn get_dev_wallet_models() -> Vec<&'static str> {
    vec![
        "anthropic/claude-3-opus",
        "anthropic/claude-3.5-sonnet",
        "openai/gpt-4-turbo",
        "meta-llama/llama-3.1-405b-instruct",
    ]
}

pub fn get_dev_preferred_model() -> &'static str {
    "anthropic/claude-3.5-sonnet"
}
