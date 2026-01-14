use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

use super::models::{
    ArbFarmCow, ArbFarmCowFork, ArbFarmCowSummary, ArbFarmExecutionMode, ArbFarmRevenue,
    ArbFarmRevenueType, ArbFarmRiskParams, ArbFarmRiskProfile, ArbFarmRiskProfileType,
    ArbFarmStrategy, ArbFarmStrategyPerformance, ArbFarmStrategyType, ArbFarmVenueType,
    CreateArbFarmCowRequest, CreateArbFarmStrategyRequest,
};

#[derive(Debug, FromRow)]
struct CowRow {
    id: Uuid,
    listing_id: Uuid,
    creator_wallet: String,
    name: String,
    description: Option<String>,
    parent_cow_id: Option<Uuid>,
    fork_count: i32,
    total_profit_generated_lamports: i64,
    total_trades: i32,
    win_rate: f64,
    creator_revenue_share_bps: i32,
    fork_revenue_share_bps: i32,
    is_public: bool,
    is_forkable: bool,
    risk_profile: JsonValue,
    inherited_engrams: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct StrategyRow {
    id: Uuid,
    cow_id: Uuid,
    name: String,
    strategy_type: String,
    venue_types: Vec<String>,
    execution_mode: String,
    risk_params: JsonValue,
    is_active: bool,
    total_trades: i32,
    successful_trades: i32,
    win_rate: f64,
    total_profit_lamports: i64,
    avg_profit_per_trade_lamports: i64,
    max_drawdown_lamports: i64,
    sharpe_ratio: Option<f64>,
    last_trade_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    #[allow(dead_code)]
    updated_at: DateTime<Utc>,
}

pub struct ArbFarmRepository;

impl ArbFarmRepository {
    pub async fn list_cows(
        pool: &PgPool,
        limit: i64,
        offset: i64,
        is_public: Option<bool>,
        is_forkable: Option<bool>,
    ) -> Result<Vec<ArbFarmCowSummary>, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT
                c.id, c.listing_id, c.name, l.description, c.creator_wallet,
                c.fork_count, c.total_profit_generated_lamports, c.total_trades, c.win_rate,
                c.is_public, c.is_forkable, c.risk_profile, c.created_at,
                l.rating, l.price_lamports, l.is_free,
                (SELECT COUNT(*) FROM arbfarm_cow_strategies WHERE cow_id = c.id) as strategy_count
            FROM arbfarm_cows c
            JOIN crossroads_listings l ON c.listing_id = l.id
            WHERE l.status = 'approved'
            "#,
        );

        if let Some(public) = is_public {
            query.push_str(&format!(" AND c.is_public = {}", public));
        }
        if let Some(forkable) = is_forkable {
            query.push_str(&format!(" AND c.is_forkable = {}", forkable));
        }

        query.push_str(&format!(
            " ORDER BY c.total_profit_generated_lamports DESC LIMIT {} OFFSET {}",
            limit, offset
        ));

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let summaries: Vec<ArbFarmCowSummary> = rows
            .iter()
            .map(|row| {
                let risk_profile: ArbFarmRiskProfile = row
                    .get::<JsonValue, _>("risk_profile")
                    .pipe(|v| serde_json::from_value(v).unwrap_or_default());
                let price_lamports: i64 = row.get("price_lamports");

                ArbFarmCowSummary {
                    id: row.get("id"),
                    listing_id: row.get("listing_id"),
                    name: row.get("name"),
                    description: row
                        .get::<Option<String>, _>("description")
                        .unwrap_or_default(),
                    creator_wallet: row.get("creator_wallet"),
                    strategy_count: row.get::<i64, _>("strategy_count") as i32,
                    venue_types: vec![],
                    risk_profile_type: risk_profile.profile_type,
                    fork_count: row.get("fork_count"),
                    total_profit_sol: row.get::<i64, _>("total_profit_generated_lamports") as f64
                        / 1_000_000_000.0,
                    win_rate: row.get::<f64, _>("win_rate") as f32,
                    price_sol: if price_lamports > 0 {
                        Some(price_lamports as f64 / 1_000_000_000.0)
                    } else {
                        None
                    },
                    is_free: row.get("is_free"),
                    is_forkable: row.get("is_forkable"),
                    rating: row.get("rating"),
                    created_at: row.get("created_at"),
                }
            })
            .collect();

        Ok(summaries)
    }

    pub async fn get_cow_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ArbFarmCow>, sqlx::Error> {
        let row: Option<CowRow> = sqlx::query_as(
            r#"
            SELECT
                c.id, c.listing_id, c.creator_wallet, c.name, c.description,
                c.parent_cow_id, c.fork_count,
                c.total_profit_generated_lamports, c.total_trades,
                c.win_rate, c.creator_revenue_share_bps, c.fork_revenue_share_bps,
                c.is_public, c.is_forkable, c.risk_profile, c.inherited_engrams,
                c.created_at, c.updated_at
            FROM arbfarm_cows c
            WHERE c.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => {
                let strategies = Self::get_strategies_for_cow(pool, id).await?;
                let risk_profile: ArbFarmRiskProfile =
                    serde_json::from_value(r.risk_profile).unwrap_or_default();

                Ok(Some(ArbFarmCow {
                    id: r.id,
                    listing_id: r.listing_id,
                    creator_wallet: r.creator_wallet,
                    name: r.name,
                    description: r.description.unwrap_or_default(),
                    strategies,
                    venue_types: vec![],
                    risk_profile,
                    parent_cow_id: r.parent_cow_id,
                    fork_count: r.fork_count,
                    total_profit_generated_lamports: r.total_profit_generated_lamports,
                    total_trades: r.total_trades,
                    win_rate: r.win_rate as f32,
                    creator_revenue_share_bps: r.creator_revenue_share_bps,
                    fork_revenue_share_bps: r.fork_revenue_share_bps,
                    is_public: r.is_public,
                    is_forkable: r.is_forkable,
                    inherited_engrams: r.inherited_engrams,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn create_cow(
        pool: &PgPool,
        request: &CreateArbFarmCowRequest,
        creator_wallet: &str,
    ) -> Result<ArbFarmCow, sqlx::Error> {
        let listing_id = Uuid::new_v4();
        let cow_id = Uuid::new_v4();

        let price_lamports = request
            .price_sol
            .map(|p| (p * 1_000_000_000.0) as i64)
            .unwrap_or(0);

        sqlx::query(
            r#"
            INSERT INTO crossroads_listings
                (id, listing_type, title, description, author, author_wallet, tags, status, price_lamports, is_free)
            VALUES ($1, 'arbfarm', $2, $3, $4, $4, $5, 'pending', $6, $7)
            "#,
        )
        .bind(listing_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(creator_wallet)
        .bind(&request.tags)
        .bind(price_lamports)
        .bind(price_lamports == 0)
        .execute(pool)
        .await?;

        let risk_profile_json = serde_json::to_value(&request.risk_profile).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO arbfarm_cows
                (id, listing_id, creator_wallet, name, description, risk_profile,
                 creator_revenue_share_bps, fork_revenue_share_bps, is_public, is_forkable)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(cow_id)
        .bind(listing_id)
        .bind(creator_wallet)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&risk_profile_json)
        .bind(request.creator_revenue_share_bps.unwrap_or(500))
        .bind(request.fork_revenue_share_bps.unwrap_or(200))
        .bind(request.is_public)
        .bind(request.is_forkable)
        .execute(pool)
        .await?;

        for strategy in &request.strategies {
            Self::create_strategy(pool, cow_id, strategy).await?;
        }

        Self::get_cow_by_id(pool, cow_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn fork_cow(
        pool: &PgPool,
        parent_id: Uuid,
        forker_wallet: &str,
        name: Option<String>,
        description: Option<String>,
        inherit_engrams: bool,
        engram_filters: Option<Vec<String>>,
    ) -> Result<(ArbFarmCow, ArbFarmCowFork), sqlx::Error> {
        let parent = Self::get_cow_by_id(pool, parent_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let fork_name = name.unwrap_or_else(|| format!("Fork of {}", parent.name));
        let fork_description = description.or_else(|| Some(parent.description.clone()));

        let forked_listing_id = Uuid::new_v4();
        let forked_cow_id = Uuid::new_v4();
        let fork_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO crossroads_listings
                (id, listing_type, title, description, author, author_wallet, tags, status, price_lamports, is_free)
            VALUES ($1, 'arbfarm', $2, $3, $4, $4, '{}', 'pending', 0, true)
            "#,
        )
        .bind(forked_listing_id)
        .bind(&fork_name)
        .bind(&fork_description)
        .bind(forker_wallet)
        .execute(pool)
        .await?;

        let risk_profile_json = serde_json::to_value(&parent.risk_profile).unwrap_or_default();
        let inherited_engrams: Vec<String> = if inherit_engrams {
            match &engram_filters {
                Some(filters) => parent
                    .inherited_engrams
                    .iter()
                    .filter(|e| filters.iter().any(|f| e.contains(f)))
                    .cloned()
                    .collect(),
                None => parent.inherited_engrams.clone(),
            }
        } else {
            vec![]
        };

        sqlx::query(
            r#"
            INSERT INTO arbfarm_cows
                (id, listing_id, creator_wallet, name, description, parent_cow_id,
                 risk_profile, inherited_engrams,
                 creator_revenue_share_bps, fork_revenue_share_bps, is_public, is_forkable)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, true)
            "#,
        )
        .bind(forked_cow_id)
        .bind(forked_listing_id)
        .bind(forker_wallet)
        .bind(&fork_name)
        .bind(&fork_description)
        .bind(parent_id)
        .bind(&risk_profile_json)
        .bind(&inherited_engrams)
        .bind(parent.creator_revenue_share_bps)
        .bind(parent.fork_revenue_share_bps)
        .execute(pool)
        .await?;

        let mut inherited_strategy_ids: Vec<Uuid> = vec![];
        for strategy in &parent.strategies {
            let new_strategy_id = Uuid::new_v4();
            inherited_strategy_ids.push(new_strategy_id);

            let risk_params_json = serde_json::to_value(&strategy.risk_params).unwrap_or_default();

            sqlx::query(
                r#"
                INSERT INTO arbfarm_cow_strategies
                    (id, cow_id, name, strategy_type, venue_types, execution_mode, risk_params, is_active)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(new_strategy_id)
            .bind(forked_cow_id)
            .bind(&strategy.name)
            .bind(format!("{:?}", strategy.strategy_type).to_lowercase())
            .bind(
                &strategy
                    .venue_types
                    .iter()
                    .map(|v| format!("{:?}", v).to_lowercase())
                    .collect::<Vec<_>>(),
            )
            .bind(format!("{:?}", strategy.execution_mode).to_lowercase())
            .bind(&risk_params_json)
            .bind(strategy.is_active)
            .execute(pool)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO arbfarm_cow_forks
                (id, parent_cow_id, forked_cow_id, forker_wallet, inherited_strategies, inherited_engrams)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(fork_id)
        .bind(parent_id)
        .bind(forked_cow_id)
        .bind(forker_wallet)
        .bind(&inherited_strategy_ids)
        .bind(&inherited_engrams)
        .execute(pool)
        .await?;

        let forked_cow = Self::get_cow_by_id(pool, forked_cow_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let fork = ArbFarmCowFork {
            id: fork_id,
            parent_cow_id: parent_id,
            forked_cow_id,
            forker_wallet: forker_wallet.to_string(),
            inherited_strategies: inherited_strategy_ids,
            inherited_engrams: inherited_engrams.clone(),
            fork_price_paid_lamports: 0,
            created_at: Utc::now(),
        };

        Ok((forked_cow, fork))
    }

    pub async fn get_strategies_for_cow(
        pool: &PgPool,
        cow_id: Uuid,
    ) -> Result<Vec<ArbFarmStrategy>, sqlx::Error> {
        let rows: Vec<StrategyRow> = sqlx::query_as(
            r#"
            SELECT
                id, cow_id, name, strategy_type, venue_types, execution_mode,
                risk_params, is_active, total_trades, successful_trades, win_rate,
                total_profit_lamports, avg_profit_per_trade_lamports, max_drawdown_lamports,
                sharpe_ratio, last_trade_at, created_at, updated_at
            FROM arbfarm_cow_strategies
            WHERE cow_id = $1
            "#,
        )
        .bind(cow_id)
        .fetch_all(pool)
        .await?;

        let strategies: Vec<ArbFarmStrategy> = rows
            .into_iter()
            .map(|r| {
                let risk_params: ArbFarmRiskParams =
                    serde_json::from_value(r.risk_params).unwrap_or_default();

                ArbFarmStrategy {
                    id: r.id,
                    name: r.name,
                    strategy_type: parse_strategy_type(&r.strategy_type),
                    venue_types: r.venue_types.iter().map(|v| parse_venue_type(v)).collect(),
                    execution_mode: parse_execution_mode(&r.execution_mode),
                    risk_params,
                    is_active: r.is_active,
                    performance: Some(ArbFarmStrategyPerformance {
                        total_trades: r.total_trades,
                        successful_trades: r.successful_trades,
                        win_rate: r.win_rate as f32,
                        total_profit_lamports: r.total_profit_lamports,
                        avg_profit_per_trade_lamports: r.avg_profit_per_trade_lamports,
                        max_drawdown_lamports: r.max_drawdown_lamports,
                        sharpe_ratio: r.sharpe_ratio.map(|s| s as f32),
                        last_trade_at: r.last_trade_at,
                    }),
                }
            })
            .collect();

        Ok(strategies)
    }

    pub async fn create_strategy(
        pool: &PgPool,
        cow_id: Uuid,
        request: &CreateArbFarmStrategyRequest,
    ) -> Result<Uuid, sqlx::Error> {
        let strategy_id = Uuid::new_v4();
        let risk_params_json = serde_json::to_value(&request.risk_params).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO arbfarm_cow_strategies
                (id, cow_id, name, strategy_type, venue_types, execution_mode, risk_params, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(strategy_id)
        .bind(cow_id)
        .bind(&request.name)
        .bind(format!("{:?}", request.strategy_type).to_lowercase())
        .bind(
            &request
                .venue_types
                .iter()
                .map(|v| format!("{:?}", v).to_lowercase())
                .collect::<Vec<_>>(),
        )
        .bind(format!("{:?}", request.execution_mode).to_lowercase())
        .bind(&risk_params_json)
        .bind(true)
        .execute(pool)
        .await?;

        Ok(strategy_id)
    }

    pub async fn get_forks_for_cow(
        pool: &PgPool,
        cow_id: Uuid,
    ) -> Result<Vec<ArbFarmCowFork>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, parent_cow_id, forked_cow_id, forker_wallet,
                inherited_strategies, inherited_engrams,
                fork_price_paid_lamports, created_at
            FROM arbfarm_cow_forks
            WHERE parent_cow_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(cow_id)
        .fetch_all(pool)
        .await?;

        let forks: Vec<ArbFarmCowFork> = rows
            .iter()
            .map(|row| ArbFarmCowFork {
                id: row.get("id"),
                parent_cow_id: row.get("parent_cow_id"),
                forked_cow_id: row.get("forked_cow_id"),
                forker_wallet: row.get("forker_wallet"),
                inherited_strategies: row.get("inherited_strategies"),
                inherited_engrams: row.get("inherited_engrams"),
                fork_price_paid_lamports: row.get("fork_price_paid_lamports"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(forks)
    }

    pub async fn get_revenue_for_cow(
        pool: &PgPool,
        cow_id: Uuid,
    ) -> Result<Vec<ArbFarmRevenue>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, cow_id, wallet_address, revenue_type, amount_lamports,
                source_trade_id, source_fork_id, period_start, period_end,
                is_distributed, distributed_at, tx_signature, created_at
            FROM arbfarm_revenue
            WHERE cow_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(cow_id)
        .fetch_all(pool)
        .await?;

        let revenues: Vec<ArbFarmRevenue> = rows
            .iter()
            .map(|row| ArbFarmRevenue {
                id: row.get("id"),
                cow_id: row.get("cow_id"),
                wallet_address: row.get("wallet_address"),
                revenue_type: parse_revenue_type(row.get::<&str, _>("revenue_type")),
                amount_lamports: row.get("amount_lamports"),
                source_trade_id: row.get("source_trade_id"),
                source_fork_id: row.get("source_fork_id"),
                period_start: row.get("period_start"),
                period_end: row.get("period_end"),
                is_distributed: row.get("is_distributed"),
                distributed_at: row.get("distributed_at"),
                tx_signature: row.get("tx_signature"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(revenues)
    }

    pub async fn get_earnings_for_wallet(
        pool: &PgPool,
        wallet: &str,
    ) -> Result<(i64, i64, i64, i64, i64, i32, i32), sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(amount_lamports), 0) as total_earnings,
                COALESCE(SUM(CASE WHEN revenue_type = 'trading_profit' THEN amount_lamports ELSE 0 END), 0) as trading_profit,
                COALESCE(SUM(CASE WHEN revenue_type = 'fork_fee' THEN amount_lamports ELSE 0 END), 0) as fork_fees,
                COALESCE(SUM(CASE WHEN revenue_type = 'creator_royalty' THEN amount_lamports ELSE 0 END), 0) as royalties,
                COALESCE(SUM(CASE WHEN is_distributed = false THEN amount_lamports ELSE 0 END), 0) as pending
            FROM arbfarm_revenue
            WHERE wallet_address = $1
            "#,
        )
        .bind(wallet)
        .fetch_one(pool)
        .await?;

        let cows_owned: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM arbfarm_cows WHERE creator_wallet = $1",
        )
        .bind(wallet)
        .fetch_one(pool)
        .await?;

        let cows_forked: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM arbfarm_cow_forks WHERE forker_wallet = $1",
        )
        .bind(wallet)
        .fetch_one(pool)
        .await?;

        Ok((
            row.get("total_earnings"),
            row.get("trading_profit"),
            row.get("fork_fees"),
            row.get("royalties"),
            row.get("pending"),
            cows_owned as i32,
            cows_forked as i32,
        ))
    }

    pub async fn get_stats(pool: &PgPool) -> Result<ArbFarmStats, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_cows,
                COUNT(*) FILTER (WHERE is_public = true) as active_cows,
                COALESCE(SUM(fork_count), 0) as total_forks,
                COALESCE(SUM(total_trades), 0) as total_trades,
                COALESCE(SUM(total_profit_generated_lamports), 0) as total_profit,
                COALESCE(AVG(win_rate), 0) as avg_win_rate
            FROM arbfarm_cows
            "#,
        )
        .fetch_one(pool)
        .await?;

        let total_distributed: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount_lamports), 0) FROM arbfarm_revenue WHERE is_distributed = true",
        )
        .fetch_one(pool)
        .await?;

        Ok(ArbFarmStats {
            total_cows: row.get::<i64, _>("total_cows") as i32,
            active_cows: row.get::<i64, _>("active_cows") as i32,
            total_forks: row.get::<i64, _>("total_forks") as i32,
            total_trades_executed: row.get::<i64, _>("total_trades") as i32,
            total_profit_generated_lamports: row.get("total_profit"),
            total_revenue_distributed_lamports: total_distributed,
            avg_win_rate: row.get::<f64, _>("avg_win_rate") as f32,
        })
    }
}

#[derive(Debug)]
pub struct ArbFarmStats {
    pub total_cows: i32,
    pub active_cows: i32,
    pub total_forks: i32,
    pub total_trades_executed: i32,
    pub total_profit_generated_lamports: i64,
    pub total_revenue_distributed_lamports: i64,
    pub avg_win_rate: f32,
}

fn parse_strategy_type(s: &str) -> ArbFarmStrategyType {
    match s.to_lowercase().as_str() {
        "dex_arb" | "dexarb" => ArbFarmStrategyType::DexArb,
        "curve_arb" | "curvearb" => ArbFarmStrategyType::CurveArb,
        "liquidation" => ArbFarmStrategyType::Liquidation,
        "jit_liquidity" | "jitliquidity" => ArbFarmStrategyType::JitLiquidity,
        "backrun" => ArbFarmStrategyType::Backrun,
        "sandwich" => ArbFarmStrategyType::Sandwich,
        "copy_trade" | "copytrade" => ArbFarmStrategyType::CopyTrade,
        _ => ArbFarmStrategyType::Custom,
    }
}

fn parse_venue_type(s: &str) -> ArbFarmVenueType {
    match s.to_lowercase().as_str() {
        "dex_amm" | "dexamm" => ArbFarmVenueType::DexAmm,
        "bonding_curve" | "bondingcurve" => ArbFarmVenueType::BondingCurve,
        "lending" => ArbFarmVenueType::Lending,
        "orderbook" => ArbFarmVenueType::Orderbook,
        _ => ArbFarmVenueType::DexAmm,
    }
}

fn parse_execution_mode(s: &str) -> ArbFarmExecutionMode {
    match s.to_lowercase().as_str() {
        "autonomous" => ArbFarmExecutionMode::Autonomous,
        "agent_directed" | "agentdirected" => ArbFarmExecutionMode::AgentDirected,
        "hybrid" => ArbFarmExecutionMode::Hybrid,
        _ => ArbFarmExecutionMode::Hybrid,
    }
}

fn parse_revenue_type(s: &str) -> ArbFarmRevenueType {
    match s.to_lowercase().as_str() {
        "trading_profit" | "tradingprofit" => ArbFarmRevenueType::TradingProfit,
        "fork_fee" | "forkfee" => ArbFarmRevenueType::ForkFee,
        "revenue_share" | "revenueshare" => ArbFarmRevenueType::RevenueShare,
        "creator_royalty" | "creatorroyalty" => ArbFarmRevenueType::CreatorRoyalty,
        _ => ArbFarmRevenueType::TradingProfit,
    }
}

trait Pipe {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
        Self: Sized;
}

impl<T> Pipe for T {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
        Self: Sized,
    {
        f(self)
    }
}

impl Default for ArbFarmRiskProfile {
    fn default() -> Self {
        Self {
            profile_type: ArbFarmRiskProfileType::Balanced,
            max_position_sol: 1.0,
            daily_loss_limit_sol: 0.5,
            max_concurrent_positions: 5,
            allowed_venue_types: vec![ArbFarmVenueType::DexAmm],
            blocked_tokens: vec![],
            custom_params: None,
        }
    }
}

impl Default for ArbFarmRiskParams {
    fn default() -> Self {
        Self {
            max_position_sol: 1.0,
            min_profit_bps: 25,
            max_slippage_bps: 100,
            max_risk_score: 50,
            daily_loss_limit_sol: None,
            require_consensus: false,
            min_consensus_agreement: None,
        }
    }
}
