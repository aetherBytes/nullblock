use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::agents::curve_metrics::{CurveMetricsCollector, DetailedCurveMetrics};
use crate::error::AppResult;
use crate::venues::curves::{HolderAnalyzer, HolderDistribution, OnChainCurveState, OnChainFetcher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Recommendation {
    StrongBuy,
    Buy,
    Hold,
    Avoid,
}

impl Recommendation {
    pub fn from_score(score: f64) -> Self {
        if score >= 80.0 {
            Recommendation::StrongBuy
        } else if score >= 60.0 {
            Recommendation::Buy
        } else if score >= 40.0 {
            Recommendation::Hold
        } else {
            Recommendation::Avoid
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Recommendation::StrongBuy => "strong_buy",
            Recommendation::Buy => "buy",
            Recommendation::Hold => "hold",
            Recommendation::Avoid => "avoid",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityScore {
    pub mint: String,
    pub venue: String,
    pub overall: f64,
    pub graduation_factor: f64,
    pub volume_factor: f64,
    pub holder_factor: f64,
    pub momentum_factor: f64,
    pub risk_penalty: f64,
    pub recommendation: Recommendation,
    pub risk_warnings: Vec<String>,
    pub positive_signals: Vec<String>,
}

impl OpportunityScore {
    pub fn is_actionable(&self) -> bool {
        matches!(
            self.recommendation,
            Recommendation::StrongBuy | Recommendation::Buy
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    pub graduation: f64,
    pub volume: f64,
    pub holders: f64,
    pub momentum: f64,
    pub risk: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            graduation: 0.30,
            volume: 0.20,
            holders: 0.20,
            momentum: 0.15,
            risk: 0.15,
        }
    }
}

impl ScoringWeights {
    pub fn aggressive() -> Self {
        Self {
            graduation: 0.40,
            volume: 0.15,
            holders: 0.15,
            momentum: 0.20,
            risk: 0.10,
        }
    }

    pub fn conservative() -> Self {
        Self {
            graduation: 0.20,
            volume: 0.20,
            holders: 0.30,
            momentum: 0.10,
            risk: 0.20,
        }
    }

    pub fn validate(&self) -> bool {
        let total = self.graduation + self.volume + self.holders + self.momentum + self.risk;
        (total - 1.0).abs() < 0.01
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringThresholds {
    pub min_graduation_progress: f64,
    pub max_graduation_progress: f64,
    pub min_volume_1h_sol: f64,
    pub min_holder_count: u32,
    pub max_top_10_concentration: f64,
    pub max_creator_holdings: f64,
    pub max_wash_trade_likelihood: f64,
    pub min_unique_buyers_1h: u32,
}

impl Default for ScoringThresholds {
    fn default() -> Self {
        Self {
            min_graduation_progress: 70.0,
            max_graduation_progress: 99.0,
            min_volume_1h_sol: 1.0,
            min_holder_count: 50,
            max_top_10_concentration: 70.0,
            max_creator_holdings: 15.0,
            max_wash_trade_likelihood: 0.6,
            min_unique_buyers_1h: 5,
        }
    }
}

pub struct CurveOpportunityScorer {
    metrics_collector: Arc<CurveMetricsCollector>,
    holder_analyzer: Arc<HolderAnalyzer>,
    on_chain_fetcher: Arc<OnChainFetcher>,
    weights: ScoringWeights,
    thresholds: ScoringThresholds,
}

impl CurveOpportunityScorer {
    pub fn new(
        metrics_collector: Arc<CurveMetricsCollector>,
        holder_analyzer: Arc<HolderAnalyzer>,
        on_chain_fetcher: Arc<OnChainFetcher>,
    ) -> Self {
        Self {
            metrics_collector,
            holder_analyzer,
            on_chain_fetcher,
            weights: ScoringWeights::default(),
            thresholds: ScoringThresholds::default(),
        }
    }

    pub fn with_weights(mut self, weights: ScoringWeights) -> Self {
        self.weights = weights;
        self
    }

    pub fn with_thresholds(mut self, thresholds: ScoringThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    pub async fn score_opportunity(
        &self,
        mint: &str,
        venue: &str,
    ) -> AppResult<OpportunityScore> {
        let metrics = self
            .metrics_collector
            .get_or_calculate_metrics(mint, venue, 300)
            .await?;

        let holders = self
            .holder_analyzer
            .get_or_analyze(mint, None, 600)
            .await?;

        let on_chain = self.on_chain_fetcher.get_bonding_curve_state(mint).await?;

        self.calculate_score(mint, venue, &metrics, &holders, &on_chain)
    }

    pub fn calculate_score(
        &self,
        mint: &str,
        venue: &str,
        metrics: &DetailedCurveMetrics,
        holders: &HolderDistribution,
        on_chain: &OnChainCurveState,
    ) -> AppResult<OpportunityScore> {
        let mut risk_warnings = Vec::new();
        let mut positive_signals = Vec::new();

        let graduation_factor = self.calculate_graduation_factor(on_chain, &mut positive_signals);
        let volume_factor = self.calculate_volume_factor(metrics, &mut positive_signals);
        let holder_factor =
            self.calculate_holder_factor(metrics, holders, &mut positive_signals, &mut risk_warnings);
        let momentum_factor = self.calculate_momentum_factor(metrics, &mut positive_signals);
        let risk_penalty =
            self.calculate_risk_penalty(metrics, holders, on_chain, &mut risk_warnings);

        let weighted_score = (graduation_factor * self.weights.graduation)
            + (volume_factor * self.weights.volume)
            + (holder_factor * self.weights.holders)
            + (momentum_factor * self.weights.momentum);

        let overall = (weighted_score * (1.0 - risk_penalty * self.weights.risk)).clamp(0.0, 100.0);

        let recommendation = Recommendation::from_score(overall);

        Ok(OpportunityScore {
            mint: mint.to_string(),
            venue: venue.to_string(),
            overall,
            graduation_factor,
            volume_factor,
            holder_factor,
            momentum_factor,
            risk_penalty,
            recommendation,
            risk_warnings,
            positive_signals,
        })
    }

    fn calculate_graduation_factor(
        &self,
        on_chain: &OnChainCurveState,
        signals: &mut Vec<String>,
    ) -> f64 {
        let progress = on_chain.graduation_progress();

        if progress < self.thresholds.min_graduation_progress {
            return 0.0;
        }

        if progress > self.thresholds.max_graduation_progress {
            signals.push("Imminent graduation".to_string());
            return 100.0;
        }

        if progress >= 95.0 {
            signals.push(format!("Near graduation: {:.1}%", progress));
            90.0 + (progress - 95.0) * 2.0
        } else if progress >= 90.0 {
            signals.push(format!("High graduation progress: {:.1}%", progress));
            70.0 + (progress - 90.0) * 4.0
        } else if progress >= 85.0 {
            50.0 + (progress - 85.0) * 4.0
        } else if progress >= 80.0 {
            30.0 + (progress - 80.0) * 4.0
        } else {
            (progress - self.thresholds.min_graduation_progress)
                / (80.0 - self.thresholds.min_graduation_progress)
                * 30.0
        }
    }

    fn calculate_volume_factor(
        &self,
        metrics: &DetailedCurveMetrics,
        signals: &mut Vec<String>,
    ) -> f64 {
        let mut score: f64 = 0.0;

        if metrics.volume_1h > 10.0 {
            signals.push(format!("High volume: {:.1} SOL/hr", metrics.volume_1h));
            score += 40.0;
        } else if metrics.volume_1h > 5.0 {
            score += 25.0;
        } else if metrics.volume_1h > self.thresholds.min_volume_1h_sol {
            score += 15.0;
        } else {
            return 0.0;
        }

        let acceleration = metrics.volume_acceleration();
        if acceleration > 1.0 {
            signals.push("Volume accelerating rapidly".to_string());
            score += 30.0;
        } else if acceleration > 0.5 {
            signals.push("Volume trending up".to_string());
            score += 20.0;
        } else if acceleration > 0.0 {
            score += 10.0;
        }

        if metrics.trade_count_1h > 50 {
            score += 20.0;
        } else if metrics.trade_count_1h > 20 {
            score += 10.0;
        }

        if metrics.avg_trade_size_sol > 0.5 {
            signals.push(format!(
                "Large avg trade size: {:.2} SOL",
                metrics.avg_trade_size_sol
            ));
            score += 10.0;
        }

        score.min(100.0)
    }

    fn calculate_holder_factor(
        &self,
        metrics: &DetailedCurveMetrics,
        holders: &HolderDistribution,
        signals: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> f64 {
        let mut score: f64 = 0.0;

        if holders.total_holders >= 100 {
            signals.push(format!("{} holders", holders.total_holders));
            score += 25.0;
        } else if holders.total_holders >= self.thresholds.min_holder_count as u32 {
            score += 15.0;
        } else {
            warnings.push(format!("Low holder count: {}", holders.total_holders));
            return 10.0;
        }

        if holders.top_10_concentration < 30.0 {
            signals.push("Well distributed holdings".to_string());
            score += 25.0;
        } else if holders.top_10_concentration < 50.0 {
            score += 15.0;
        } else if holders.top_10_concentration > self.thresholds.max_top_10_concentration {
            warnings.push(format!(
                "High concentration: top 10 hold {:.1}%",
                holders.top_10_concentration
            ));
        }

        if holders.creator_holdings_percent < 5.0 {
            score += 20.0;
        } else if holders.creator_holdings_percent < 10.0 {
            score += 10.0;
        } else if holders.creator_holdings_percent > self.thresholds.max_creator_holdings {
            warnings.push(format!(
                "Creator holds {:.1}%",
                holders.creator_holdings_percent
            ));
        }

        if holders.gini_coefficient < 0.5 {
            signals.push("Fair distribution (low Gini)".to_string());
            score += 15.0;
        } else if holders.gini_coefficient < 0.7 {
            score += 10.0;
        }

        if metrics.holder_growth_1h > 10 {
            signals.push(format!("+{} holders in 1h", metrics.holder_growth_1h));
            score += 15.0;
        } else if metrics.holder_growth_1h > 0 {
            score += 10.0;
        } else if metrics.holder_growth_1h < -5 {
            warnings.push("Holders declining".to_string());
        }

        score.min(100.0)
    }

    fn calculate_momentum_factor(
        &self,
        metrics: &DetailedCurveMetrics,
        signals: &mut Vec<String>,
    ) -> f64 {
        let mut score: f64 = 0.0;

        if metrics.price_momentum_1h > 20.0 {
            signals.push(format!("+{:.1}% price 1h", metrics.price_momentum_1h));
            score += 35.0;
        } else if metrics.price_momentum_1h > 10.0 {
            signals.push(format!("+{:.1}% price 1h", metrics.price_momentum_1h));
            score += 25.0;
        } else if metrics.price_momentum_1h > 5.0 {
            score += 15.0;
        } else if metrics.price_momentum_1h > 0.0 {
            score += 10.0;
        } else if metrics.price_momentum_1h < -10.0 {
            return 0.0;
        }

        if metrics.buy_sell_ratio_1h > 2.0 {
            signals.push(format!(
                "Strong buy pressure: {:.1}x",
                metrics.buy_sell_ratio_1h
            ));
            score += 30.0;
        } else if metrics.buy_sell_ratio_1h > 1.5 {
            signals.push("Healthy buy pressure".to_string());
            score += 20.0;
        } else if metrics.buy_sell_ratio_1h > 1.2 {
            score += 10.0;
        } else if metrics.buy_sell_ratio_1h < 0.8 {
            return score.max(10.0);
        }

        let velocity_bonus = (metrics.volume_velocity / 5.0).min(20.0).max(0.0);
        score += velocity_bonus;

        if metrics.unique_buyers_1h > 15 {
            signals.push(format!("{} unique buyers 1h", metrics.unique_buyers_1h));
            score += 15.0;
        } else if metrics.unique_buyers_1h >= self.thresholds.min_unique_buyers_1h {
            score += 10.0;
        }

        score.min(100.0)
    }

    fn calculate_risk_penalty(
        &self,
        metrics: &DetailedCurveMetrics,
        holders: &HolderDistribution,
        on_chain: &OnChainCurveState,
        warnings: &mut Vec<String>,
    ) -> f64 {
        let mut penalty: f64 = 0.0;

        if holders.wash_trade_likelihood > self.thresholds.max_wash_trade_likelihood {
            warnings.push(format!(
                "Wash trading suspected: {:.0}%",
                holders.wash_trade_likelihood * 100.0
            ));
            penalty += 0.3;
        } else if holders.wash_trade_likelihood > 0.3 {
            warnings.push("Minor wash trading signals".to_string());
            penalty += 0.1;
        }

        if on_chain.real_sol_reserves < 5_000_000_000 {
            let reserves_sol = on_chain.real_sol_reserves as f64 / 1e9;
            warnings.push(format!("Low liquidity: {:.1} SOL", reserves_sol));
            penalty += 0.2;
        }

        if metrics.market_cap_sol < 10.0 {
            warnings.push(format!("Low market cap: {:.1} SOL", metrics.market_cap_sol));
            penalty += 0.15;
        }

        if holders.top_10_concentration > 80.0 {
            penalty += 0.25;
        } else if holders.top_10_concentration > 70.0 {
            penalty += 0.15;
        }

        if metrics.volume_1h < 0.5 && metrics.trade_count_1h < 5 {
            warnings.push("Very low activity".to_string());
            penalty += 0.2;
        }

        if on_chain.graduation_progress() > 98.0 && metrics.volume_1h < 2.0 {
            warnings.push("Near graduation but low volume".to_string());
            penalty += 0.15;
        }

        if holders.creator_holdings_percent > 20.0 {
            warnings.push("Creator holds significant supply".to_string());
            penalty += 0.2;
        }

        penalty.min(1.0)
    }

    pub async fn rank_opportunities(
        &self,
        candidates: &[(String, String)],
        limit: usize,
    ) -> Vec<OpportunityScore> {
        let mut scores = Vec::new();

        for (mint, venue) in candidates {
            match self.score_opportunity(mint, venue).await {
                Ok(score) => {
                    if score.overall > 0.0 {
                        scores.push(score);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to score {}: {}", mint, e);
                }
            }
        }

        scores.sort_by(|a, b| b.overall.partial_cmp(&a.overall).unwrap());
        scores.truncate(limit);

        scores
    }

    pub fn get_weights(&self) -> &ScoringWeights {
        &self.weights
    }

    pub fn get_thresholds(&self) -> &ScoringThresholds {
        &self.thresholds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn mock_metrics() -> DetailedCurveMetrics {
        DetailedCurveMetrics {
            mint: "test".to_string(),
            venue: "pump_fun".to_string(),
            volume_1h: 15.0,
            volume_24h: 200.0,
            volume_velocity: 8.0,
            trade_count_1h: 45,
            trade_count_24h: 500,
            unique_buyers_1h: 20,
            unique_buyers_24h: 150,
            holder_count: 120,
            holder_growth_1h: 12,
            holder_growth_24h: 45,
            top_10_concentration: 35.0,
            top_20_concentration: 50.0,
            creator_holdings_percent: 4.0,
            price_momentum_1h: 15.0,
            price_momentum_24h: 45.0,
            buy_sell_ratio_1h: 1.8,
            avg_trade_size_sol: 0.35,
            graduation_progress: 88.0,
            market_cap_sol: 65.0,
            liquidity_depth_sol: 25.0,
            last_updated: Utc::now(),
        }
    }

    fn mock_holders() -> HolderDistribution {
        HolderDistribution {
            mint: "test".to_string(),
            total_holders: 120,
            total_supply: 1_000_000_000,
            circulating_supply: 1_000_000_000,
            top_10_holders: vec![],
            top_10_concentration: 35.0,
            top_20_concentration: 50.0,
            top_50_concentration: 70.0,
            creator_address: None,
            creator_holdings_percent: 4.0,
            gini_coefficient: 0.55,
            unique_wallets_24h: 150,
            new_holders_24h: 45,
            wash_trade_likelihood: 0.1,
            analyzed_at: Utc::now(),
        }
    }

    fn mock_on_chain() -> OnChainCurveState {
        OnChainCurveState {
            mint: "test".to_string(),
            bonding_curve_address: "curve".to_string(),
            associated_bonding_curve: "associated_curve".to_string(),
            virtual_token_reserves: 800_000_000_000,
            virtual_sol_reserves: 30_000_000_000,
            real_token_reserves: 200_000_000_000,
            real_sol_reserves: 20_000_000_000,
            token_total_supply: 1_000_000_000_000,
            is_complete: false,
            creator: "creator".to_string(),
            created_slot: 0,
        }
    }

    #[test]
    fn test_recommendation_from_score() {
        assert_eq!(Recommendation::from_score(85.0), Recommendation::StrongBuy);
        assert_eq!(Recommendation::from_score(70.0), Recommendation::Buy);
        assert_eq!(Recommendation::from_score(50.0), Recommendation::Hold);
        assert_eq!(Recommendation::from_score(30.0), Recommendation::Avoid);
    }

    #[test]
    fn test_scoring_weights_validation() {
        let weights = ScoringWeights::default();
        assert!(weights.validate());

        let aggressive = ScoringWeights::aggressive();
        assert!(aggressive.validate());

        let conservative = ScoringWeights::conservative();
        assert!(conservative.validate());
    }

    #[test]
    fn test_healthy_opportunity_scoring() {
        let metrics = mock_metrics();
        let holders = mock_holders();
        let on_chain = mock_on_chain();

        let scorer = CurveOpportunityScorer {
            metrics_collector: Arc::new(CurveMetricsCollector::new_mock()),
            holder_analyzer: Arc::new(HolderAnalyzer::new_mock()),
            on_chain_fetcher: Arc::new(OnChainFetcher::new_mock()),
            weights: ScoringWeights::default(),
            thresholds: ScoringThresholds::default(),
        };

        let score = scorer
            .calculate_score("test", "pump_fun", &metrics, &holders, &on_chain)
            .unwrap();

        assert!(
            score.overall >= 60.0,
            "Healthy opportunity should score >= 60, got {}",
            score.overall
        );
        assert!(score.is_actionable());
        assert!(!score.positive_signals.is_empty());
    }

    #[test]
    fn test_risky_opportunity_scoring() {
        let mut metrics = mock_metrics();
        metrics.volume_1h = 0.3;
        metrics.trade_count_1h = 3;
        metrics.holder_growth_1h = -5;

        let mut holders = mock_holders();
        holders.total_holders = 25;
        holders.top_10_concentration = 85.0;
        holders.wash_trade_likelihood = 0.7;

        let mut on_chain = mock_on_chain();
        on_chain.real_sol_reserves = 2_000_000_000;

        let scorer = CurveOpportunityScorer {
            metrics_collector: Arc::new(CurveMetricsCollector::new_mock()),
            holder_analyzer: Arc::new(HolderAnalyzer::new_mock()),
            on_chain_fetcher: Arc::new(OnChainFetcher::new_mock()),
            weights: ScoringWeights::default(),
            thresholds: ScoringThresholds::default(),
        };

        let score = scorer
            .calculate_score("test", "pump_fun", &metrics, &holders, &on_chain)
            .unwrap();

        assert!(
            score.overall < 50.0,
            "Risky opportunity should score < 50, got {}",
            score.overall
        );
        assert!(!score.risk_warnings.is_empty());
        assert_eq!(score.recommendation, Recommendation::Avoid);
    }
}
