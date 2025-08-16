"""
Data Analyzer Component

Handles the core data analysis logic for the Information Gathering Agent.
Provides trend analysis, statistical analysis, and insight generation.
"""

import logging
import numpy as np
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
from datetime import datetime, timedelta
import statistics

logger = logging.getLogger(__name__)

@dataclass
class AnalysisMetrics:
    """Statistical metrics for data analysis"""
    mean: float
    median: float
    std_dev: float
    variance: float
    min_value: float
    max_value: float
    trend_direction: str  # 'up', 'down', 'sideways'
    volatility: float
    
class DataAnalyzer:
    """
    Core data analysis engine for processing various data types
    """
    
    def __init__(self):
        self.analysis_history: Dict[str, List[Dict[str, Any]]] = {}
        logger.info("DataAnalyzer initialized")
    
    async def analyze(self, raw_data: Dict[str, Any], analysis_type: str, context: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Main analysis method that routes to specific analysis functions
        
        Args:
            raw_data: Raw data from data sources
            analysis_type: Type of analysis ('trend', 'pattern', 'anomaly', 'correlation')
            context: Additional context for analysis
            
        Returns:
            Analysis results with insights and recommendations
        """
        try:
            logger.info(f"Starting {analysis_type} analysis")
            
            if analysis_type == "trend":
                return await self._analyze_trends(raw_data, context)
            elif analysis_type == "pattern":
                return await self._analyze_patterns(raw_data, context)
            elif analysis_type == "anomaly":
                return await self._detect_anomalies(raw_data, context)
            elif analysis_type == "correlation":
                return await self._analyze_correlations(raw_data, context)
            else:
                return await self._general_analysis(raw_data, context)
                
        except Exception as e:
            logger.error(f"Error in analysis: {e}")
            return {
                "insights": [f"Analysis failed: {str(e)}"],
                "confidence": 0.0,
                "recommendations": ["Unable to complete analysis due to error"]
            }
    
    async def _analyze_trends(self, data: Dict[str, Any], context: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """Analyze trends in time series data"""
        insights = []
        recommendations = []
        confidence = 0.0
        
        try:
            # Extract time series data
            time_series = self._extract_time_series(data)
            
            if not time_series:
                return {
                    "insights": ["No time series data available for trend analysis"],
                    "confidence": 0.0,
                    "recommendations": ["Ensure data contains time-indexed values"]
                }
            
            # Calculate basic metrics
            metrics = self._calculate_metrics(time_series)
            
            # Determine trend direction
            if len(time_series) >= 2:
                recent_trend = self._calculate_trend_direction(time_series[-10:])  # Last 10 points
                overall_trend = self._calculate_trend_direction(time_series)
                
                insights.append(f"Recent trend: {recent_trend}")
                insights.append(f"Overall trend: {overall_trend}")
                
                # Volatility analysis
                volatility = metrics.volatility
                if volatility > 0.1:
                    insights.append(f"High volatility detected: {volatility:.2%}")
                    recommendations.append("Consider risk management strategies due to high volatility")
                elif volatility < 0.02:
                    insights.append(f"Low volatility detected: {volatility:.2%}")
                    recommendations.append("Stable conditions, suitable for conservative strategies")
                
                # Trend strength
                trend_strength = self._calculate_trend_strength(time_series)
                insights.append(f"Trend strength: {trend_strength:.2f}")
                
                confidence = min(0.9, trend_strength)
            
            # Context-specific analysis
            if context and context.get("analysis_goal") == "market_trends":
                market_insights = self._analyze_market_context(time_series, context)
                insights.extend(market_insights)
                
        except Exception as e:
            logger.error(f"Error in trend analysis: {e}")
            insights.append(f"Trend analysis error: {str(e)}")
        
        return {
            "insights": insights,
            "confidence": confidence,
            "recommendations": recommendations,
            "metrics": metrics.__dict__ if 'metrics' in locals() else {}
        }
    
    async def _analyze_patterns(self, data: Dict[str, Any], context: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """Analyze patterns in the data"""
        insights = []
        recommendations = []
        confidence = 0.0
        
        try:
            # Extract numerical data for pattern analysis
            numerical_data = self._extract_numerical_data(data)
            
            if not numerical_data:
                return {
                    "insights": ["No numerical data available for pattern analysis"],
                    "confidence": 0.0,
                    "recommendations": ["Ensure data contains numerical values"]
                }
            
            # Detect recurring patterns
            patterns = self._detect_recurring_patterns(numerical_data)
            if patterns:
                insights.extend([f"Pattern detected: {pattern}" for pattern in patterns])
                confidence += 0.3
            
            # Detect cyclical behavior
            cycles = self._detect_cycles(numerical_data)
            if cycles:
                insights.extend([f"Cyclical behavior: {cycle}" for cycle in cycles])
                confidence += 0.2
                recommendations.append("Consider timing strategies based on detected cycles")
            
            # Support and resistance levels
            if context and context.get("analysis_goal") == "defi_opportunities":
                levels = self._find_support_resistance(numerical_data)
                if levels:
                    insights.append(f"Support levels: {levels['support']}")
                    insights.append(f"Resistance levels: {levels['resistance']}")
                    recommendations.append("Monitor these levels for potential trading opportunities")
                    confidence += 0.2
            
        except Exception as e:
            logger.error(f"Error in pattern analysis: {e}")
            insights.append(f"Pattern analysis error: {str(e)}")
        
        return {
            "insights": insights,
            "confidence": min(confidence, 0.9),
            "recommendations": recommendations
        }
    
    async def _detect_anomalies(self, data: Dict[str, Any], context: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """Detect anomalies in the data"""
        insights = []
        recommendations = []
        confidence = 0.0
        
        try:
            time_series = self._extract_time_series(data)
            
            if len(time_series) < 10:
                return {
                    "insights": ["Insufficient data for anomaly detection"],
                    "confidence": 0.0,
                    "recommendations": ["Collect more data points for reliable anomaly detection"]
                }
            
            # Statistical anomaly detection
            anomalies = self._statistical_anomaly_detection(time_series)
            
            if anomalies:
                insights.append(f"Found {len(anomalies)} statistical anomalies")
                insights.extend([f"Anomaly at index {idx}: {val}" for idx, val in anomalies[:5]])  # Top 5
                
                # Analyze anomaly patterns
                if len(anomalies) > 1:
                    anomaly_pattern = self._analyze_anomaly_patterns(anomalies)
                    insights.append(f"Anomaly pattern: {anomaly_pattern}")
                
                recommendations.append("Investigate the causes of detected anomalies")
                confidence = 0.7
            else:
                insights.append("No significant anomalies detected")
                confidence = 0.8
                
        except Exception as e:
            logger.error(f"Error in anomaly detection: {e}")
            insights.append(f"Anomaly detection error: {str(e)}")
        
        return {
            "insights": insights,
            "confidence": confidence,
            "recommendations": recommendations
        }
    
    async def _analyze_correlations(self, data: Dict[str, Any], context: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """Analyze correlations between different data series"""
        insights = []
        recommendations = []
        confidence = 0.0
        
        try:
            # Extract multiple time series for correlation analysis
            series_data = self._extract_multiple_series(data)
            
            if len(series_data) < 2:
                return {
                    "insights": ["Need at least 2 data series for correlation analysis"],
                    "confidence": 0.0,
                    "recommendations": ["Provide multiple data series for correlation analysis"]
                }
            
            # Calculate correlations between all pairs
            correlations = self._calculate_correlations(series_data)
            
            # Analyze strong correlations
            strong_correlations = [(pair, corr) for pair, corr in correlations.items() if abs(corr) > 0.7]
            
            if strong_correlations:
                insights.extend([f"Strong correlation between {pair[0]} and {pair[1]}: {corr:.3f}" 
                               for pair, corr in strong_correlations])
                recommendations.append("Consider these correlations in strategy development")
                confidence = 0.8
            
            # Identify leading indicators
            leading_indicators = self._find_leading_indicators(series_data)
            if leading_indicators:
                insights.extend([f"Leading indicator: {indicator}" for indicator in leading_indicators])
                recommendations.append("Use leading indicators for predictive strategies")
                
        except Exception as e:
            logger.error(f"Error in correlation analysis: {e}")
            insights.append(f"Correlation analysis error: {str(e)}")
        
        return {
            "insights": insights,
            "confidence": confidence,
            "recommendations": recommendations
        }
    
    async def _general_analysis(self, data: Dict[str, Any], context: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """General purpose analysis for unspecified analysis types"""
        insights = []
        recommendations = []
        
        # Basic data summary
        data_summary = self._summarize_data(data)
        insights.extend(data_summary)
        
        # Generic recommendations
        recommendations.extend([
            "Consider more specific analysis types for detailed insights",
            "Monitor data quality and completeness",
            "Establish baseline metrics for comparison"
        ])
        
        return {
            "insights": insights,
            "confidence": 0.5,
            "recommendations": recommendations
        }
    
    def _extract_time_series(self, data: Dict[str, Any]) -> List[float]:
        """Extract time series data from various data formats"""
        try:
            # Try common time series keys
            for key in ['prices', 'values', 'data', 'time_series', 'price_history']:
                if key in data and isinstance(data[key], list):
                    return [float(x) for x in data[key] if isinstance(x, (int, float))]
            
            # Try extracting from nested structures
            if 'result' in data and isinstance(data['result'], list):
                return [float(x) for x in data['result'] if isinstance(x, (int, float))]
                
            return []
        except Exception as e:
            logger.error(f"Error extracting time series: {e}")
            return []
    
    def _extract_numerical_data(self, data: Dict[str, Any]) -> List[float]:
        """Extract all numerical data from the data structure"""
        numerical_values = []
        
        def extract_numbers(obj):
            if isinstance(obj, (int, float)):
                numerical_values.append(float(obj))
            elif isinstance(obj, list):
                for item in obj:
                    extract_numbers(item)
            elif isinstance(obj, dict):
                for value in obj.values():
                    extract_numbers(value)
        
        extract_numbers(data)
        return numerical_values
    
    def _calculate_metrics(self, time_series: List[float]) -> AnalysisMetrics:
        """Calculate statistical metrics for time series data"""
        if not time_series:
            return AnalysisMetrics(0, 0, 0, 0, 0, 0, "unknown", 0)
        
        mean_val = statistics.mean(time_series)
        median_val = statistics.median(time_series)
        std_dev = statistics.stdev(time_series) if len(time_series) > 1 else 0
        variance = statistics.variance(time_series) if len(time_series) > 1 else 0
        min_val = min(time_series)
        max_val = max(time_series)
        
        # Calculate trend direction
        if len(time_series) >= 2:
            trend_direction = "up" if time_series[-1] > time_series[0] else "down"
            if abs(time_series[-1] - time_series[0]) / time_series[0] < 0.02:  # Less than 2% change
                trend_direction = "sideways"
        else:
            trend_direction = "unknown"
        
        # Calculate volatility (coefficient of variation)
        volatility = std_dev / mean_val if mean_val != 0 else 0
        
        return AnalysisMetrics(
            mean=mean_val,
            median=median_val,
            std_dev=std_dev,
            variance=variance,
            min_value=min_val,
            max_value=max_val,
            trend_direction=trend_direction,
            volatility=volatility
        )
    
    def _calculate_trend_direction(self, time_series: List[float]) -> str:
        """Calculate trend direction using linear regression"""
        if len(time_series) < 2:
            return "unknown"
        
        x = list(range(len(time_series)))
        y = time_series
        
        # Simple linear regression
        n = len(x)
        sum_x = sum(x)
        sum_y = sum(y)
        sum_xy = sum(x[i] * y[i] for i in range(n))
        sum_x2 = sum(x[i] * x[i] for i in range(n))
        
        slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
        
        if slope > 0.01:
            return "up"
        elif slope < -0.01:
            return "down"
        else:
            return "sideways"
    
    def _calculate_trend_strength(self, time_series: List[float]) -> float:
        """Calculate the strength of the trend (0-1)"""
        if len(time_series) < 3:
            return 0.0
        
        # Calculate R-squared for trend line
        x = list(range(len(time_series)))
        y = time_series
        
        try:
            correlation = np.corrcoef(x, y)[0, 1]
            r_squared = correlation ** 2
            return r_squared
        except:
            return 0.0
    
    def _detect_recurring_patterns(self, data: List[float]) -> List[str]:
        """Detect recurring patterns in numerical data"""
        patterns = []
        
        if len(data) < 6:
            return patterns
        
        # Simple pattern detection - look for repeating sequences
        for pattern_length in range(2, min(6, len(data) // 3)):
            for start in range(len(data) - pattern_length * 2):
                pattern = data[start:start + pattern_length]
                next_pattern = data[start + pattern_length:start + pattern_length * 2]
                
                # Check if patterns are similar (within 5% tolerance)
                if self._patterns_similar(pattern, next_pattern, tolerance=0.05):
                    patterns.append(f"Repeating sequence of length {pattern_length}")
                    break
        
        return list(set(patterns))  # Remove duplicates
    
    def _patterns_similar(self, pattern1: List[float], pattern2: List[float], tolerance: float = 0.05) -> bool:
        """Check if two patterns are similar within tolerance"""
        if len(pattern1) != len(pattern2):
            return False
        
        for v1, v2 in zip(pattern1, pattern2):
            if abs(v1 - v2) / max(abs(v1), abs(v2), 1e-8) > tolerance:
                return False
        
        return True
    
    def _detect_cycles(self, data: List[float]) -> List[str]:
        """Detect cyclical behavior in data"""
        cycles = []
        
        if len(data) < 10:
            return cycles
        
        # Simple cycle detection - look for peaks and troughs
        peaks = []
        troughs = []
        
        for i in range(1, len(data) - 1):
            if data[i] > data[i-1] and data[i] > data[i+1]:
                peaks.append(i)
            elif data[i] < data[i-1] and data[i] < data[i+1]:
                troughs.append(i)
        
        # Analyze peak intervals
        if len(peaks) >= 3:
            intervals = [peaks[i+1] - peaks[i] for i in range(len(peaks)-1)]
            avg_interval = statistics.mean(intervals)
            cycles.append(f"Peak cycle every ~{avg_interval:.1f} periods")
        
        return cycles
    
    def _find_support_resistance(self, data: List[float]) -> Dict[str, List[float]]:
        """Find support and resistance levels"""
        if len(data) < 5:
            return {"support": [], "resistance": []}
        
        # Find local minima (support) and maxima (resistance)
        support_levels = []
        resistance_levels = []
        
        for i in range(2, len(data) - 2):
            # Support level (local minimum)
            if all(data[i] <= data[j] for j in range(i-2, i+3)):
                support_levels.append(data[i])
            
            # Resistance level (local maximum)
            if all(data[i] >= data[j] for j in range(i-2, i+3)):
                resistance_levels.append(data[i])
        
        return {
            "support": sorted(set(support_levels))[:3],  # Top 3 support levels
            "resistance": sorted(set(resistance_levels), reverse=True)[:3]  # Top 3 resistance levels
        }
    
    def _statistical_anomaly_detection(self, data: List[float]) -> List[tuple]:
        """Detect statistical anomalies using z-score method"""
        if len(data) < 3:
            return []
        
        mean_val = statistics.mean(data)
        std_dev = statistics.stdev(data)
        
        if std_dev == 0:
            return []
        
        anomalies = []
        threshold = 2.5  # Z-score threshold
        
        for i, value in enumerate(data):
            z_score = abs(value - mean_val) / std_dev
            if z_score > threshold:
                anomalies.append((i, value))
        
        return anomalies
    
    def _analyze_anomaly_patterns(self, anomalies: List[tuple]) -> str:
        """Analyze patterns in detected anomalies"""
        if len(anomalies) < 2:
            return "Isolated anomalies"
        
        # Check if anomalies are clustered
        indices = [idx for idx, _ in anomalies]
        intervals = [indices[i+1] - indices[i] for i in range(len(indices)-1)]
        
        if max(intervals) - min(intervals) <= 2:
            return "Clustered anomalies"
        elif statistics.mean(intervals) < 5:
            return "Frequent anomalies"
        else:
            return "Sporadic anomalies"
    
    def _extract_multiple_series(self, data: Dict[str, Any]) -> Dict[str, List[float]]:
        """Extract multiple time series for correlation analysis"""
        series = {}
        
        # Look for multiple series in the data
        for key, value in data.items():
            if isinstance(value, list) and all(isinstance(x, (int, float)) for x in value[:5]):
                series[key] = [float(x) for x in value]
        
        return series
    
    def _calculate_correlations(self, series_data: Dict[str, List[float]]) -> Dict[tuple, float]:
        """Calculate correlations between all pairs of series"""
        correlations = {}
        series_names = list(series_data.keys())
        
        for i in range(len(series_names)):
            for j in range(i + 1, len(series_names)):
                name1, name2 = series_names[i], series_names[j]
                series1, series2 = series_data[name1], series_data[name2]
                
                # Ensure series are same length
                min_length = min(len(series1), len(series2))
                if min_length > 1:
                    corr = np.corrcoef(series1[:min_length], series2[:min_length])[0, 1]
                    if not np.isnan(corr):
                        correlations[(name1, name2)] = corr
        
        return correlations
    
    def _find_leading_indicators(self, series_data: Dict[str, List[float]]) -> List[str]:
        """Find series that lead others (lagged correlation analysis)"""
        leading_indicators = []
        series_names = list(series_data.keys())
        
        for i in range(len(series_names)):
            for j in range(len(series_names)):
                if i != j:
                    name1, name2 = series_names[i], series_names[j]
                    series1, series2 = series_data[name1], series_data[name2]
                    
                    # Check if series1 leads series2 by calculating correlation with lag
                    if self._check_leading_relationship(series1, series2):
                        if name1 not in leading_indicators:
                            leading_indicators.append(name1)
        
        return leading_indicators
    
    def _check_leading_relationship(self, series1: List[float], series2: List[float], max_lag: int = 5) -> bool:
        """Check if series1 leads series2"""
        if len(series1) < max_lag + 2 or len(series2) < max_lag + 2:
            return False
        
        best_correlation = 0
        
        for lag in range(1, max_lag + 1):
            if len(series1) > lag and len(series2) > lag:
                try:
                    # Correlate series1[:-lag] with series2[lag:]
                    corr = np.corrcoef(series1[:-lag], series2[lag:])[0, 1]
                    if not np.isnan(corr) and abs(corr) > abs(best_correlation):
                        best_correlation = corr
                except:
                    continue
        
        return abs(best_correlation) > 0.6  # Strong leading relationship threshold
    
    def _summarize_data(self, data: Dict[str, Any]) -> List[str]:
        """Generate a basic summary of the data"""
        summary = []
        
        # Count different data types
        num_keys = len(data.keys()) if isinstance(data, dict) else 0
        summary.append(f"Data contains {num_keys} main keys")
        
        # Identify data types
        numerical_data = self._extract_numerical_data(data)
        if numerical_data:
            summary.append(f"Found {len(numerical_data)} numerical values")
            summary.append(f"Value range: {min(numerical_data):.2f} to {max(numerical_data):.2f}")
        
        time_series = self._extract_time_series(data)
        if time_series:
            summary.append(f"Time series data with {len(time_series)} points")
        
        return summary
    
    def _analyze_market_context(self, time_series: List[float], context: Dict[str, Any]) -> List[str]:
        """Analyze data specifically in market context"""
        insights = []
        
        if not time_series:
            return insights
        
        # Price movement analysis
        if len(time_series) >= 2:
            price_change = (time_series[-1] - time_series[0]) / time_series[0]
            insights.append(f"Total price change: {price_change:.2%}")
            
            if abs(price_change) > 0.1:
                insights.append("Significant price movement detected")
            
        # Volatility in market context
        if len(time_series) > 5:
            recent_volatility = np.std(time_series[-5:]) / np.mean(time_series[-5:])
            overall_volatility = np.std(time_series) / np.mean(time_series)
            
            if recent_volatility > overall_volatility * 1.5:
                insights.append("Recent volatility spike detected")
            
        return insights