"""
Pattern Detector Component

Advanced pattern detection and recognition for the Information Gathering Agent.
Focuses on identifying actionable patterns in multi-source data streams.
"""

import logging
import numpy as np
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass
from datetime import datetime, timedelta
import asyncio

logger = logging.getLogger(__name__)

@dataclass
class Pattern:
    """Represents a detected pattern"""
    pattern_type: str  # 'trend', 'cycle', 'breakout', 'support_resistance', 'divergence'
    confidence: float  # 0.0-1.0
    description: str
    data_source: str
    start_time: Optional[datetime] = None
    end_time: Optional[datetime] = None
    parameters: Dict[str, Any] = None
    
@dataclass
class Anomaly:
    """Represents a detected anomaly"""
    anomaly_type: str  # 'statistical', 'behavioral', 'volume', 'price'
    severity: str  # 'low', 'medium', 'high'
    description: str
    value: float
    expected_value: float
    deviation_score: float
    timestamp: datetime

class PatternDetector:
    """
    Advanced pattern detection engine for multi-source data analysis
    """
    
    def __init__(self):
        self.pattern_history: Dict[str, List[Pattern]] = {}
        self.anomaly_history: Dict[str, List[Anomaly]] = {}
        self.baseline_metrics: Dict[str, Dict[str, float]] = {}
        logger.info("PatternDetector initialized")
    
    async def detect_patterns(self, data: Dict[str, Any], analysis_type: str) -> Dict[str, Any]:
        """
        Main pattern detection method
        
        Args:
            data: Raw data from various sources
            analysis_type: Type of analysis being performed
            
        Returns:
            Dictionary containing detected patterns and anomalies
        """
        try:
            patterns = []
            anomalies = []
            
            # Extract time series data
            time_series_data = self._extract_time_series_data(data)
            
            if time_series_data:
                # Detect various pattern types
                if analysis_type in ['trend', 'pattern', 'general']:
                    patterns.extend(await self._detect_trend_patterns(time_series_data))
                    patterns.extend(await self._detect_cyclical_patterns(time_series_data))
                    patterns.extend(await self._detect_breakout_patterns(time_series_data))
                    patterns.extend(await self._detect_support_resistance_patterns(time_series_data))
                
                # Detect anomalies
                anomalies.extend(await self._detect_statistical_anomalies(time_series_data))
                anomalies.extend(await self._detect_behavioral_anomalies(time_series_data))
            
            # Store patterns for historical analysis
            data_source = data.get('source', 'unknown')
            self._store_patterns(data_source, patterns)
            self._store_anomalies(data_source, anomalies)
            
            return {
                'patterns': [pattern.__dict__ for pattern in patterns],
                'anomalies': [anomaly.__dict__ for anomaly in anomalies],
                'pattern_summary': self._generate_pattern_summary(patterns),
                'anomaly_summary': self._generate_anomaly_summary(anomalies)
            }
            
        except Exception as e:
            logger.error(f"Error in pattern detection: {e}")
            return {'patterns': [], 'anomalies': [], 'error': str(e)}
    
    async def update_patterns(self):
        """Update pattern detection models with recent data"""
        try:
            # Update baseline metrics for anomaly detection
            await self._update_baseline_metrics()
            
            # Clean old patterns and anomalies
            await self._cleanup_old_data()
            
            logger.debug("Pattern detection models updated")
            
        except Exception as e:
            logger.error(f"Error updating patterns: {e}")
    
    async def _detect_trend_patterns(self, time_series_data: Dict[str, List[float]]) -> List[Pattern]:
        """Detect trend patterns in time series data"""
        patterns = []
        
        for source, data in time_series_data.items():
            if len(data) < 5:
                continue
                
            try:
                # Linear trend detection
                trend_strength, trend_direction = self._calculate_trend_metrics(data)
                
                if trend_strength > 0.7:  # Strong trend
                    patterns.append(Pattern(
                        pattern_type='trend',
                        confidence=trend_strength,
                        description=f"Strong {trend_direction} trend detected",
                        data_source=source,
                        parameters={
                            'direction': trend_direction,
                            'strength': trend_strength,
                            'duration': len(data)
                        }
                    ))
                
                # Trend reversal detection
                reversal_signals = self._detect_trend_reversals(data)
                for signal in reversal_signals:
                    patterns.append(Pattern(
                        pattern_type='trend_reversal',
                        confidence=signal['confidence'],
                        description=f"Potential trend reversal: {signal['description']}",
                        data_source=source,
                        parameters=signal
                    ))
                    
            except Exception as e:
                logger.error(f"Error detecting trend patterns for {source}: {e}")
        
        return patterns
    
    async def _detect_cyclical_patterns(self, time_series_data: Dict[str, List[float]]) -> List[Pattern]:
        """Detect cyclical and seasonal patterns"""
        patterns = []
        
        for source, data in time_series_data.items():
            if len(data) < 20:  # Need sufficient data for cycle detection
                continue
                
            try:
                # Detect repeating cycles
                cycles = self._find_cycles(data)
                
                for cycle in cycles:
                    if cycle['confidence'] > 0.6:
                        patterns.append(Pattern(
                            pattern_type='cycle',
                            confidence=cycle['confidence'],
                            description=f"Cyclical pattern: period {cycle['period']}, amplitude {cycle['amplitude']:.2f}",
                            data_source=source,
                            parameters=cycle
                        ))
                
                # Detect seasonal patterns (if timestamp data available)
                seasonal_patterns = self._detect_seasonal_patterns(data)
                patterns.extend(seasonal_patterns)
                
            except Exception as e:
                logger.error(f"Error detecting cyclical patterns for {source}: {e}")
        
        return patterns
    
    async def _detect_breakout_patterns(self, time_series_data: Dict[str, List[float]]) -> List[Pattern]:
        """Detect breakout patterns from consolidation ranges"""
        patterns = []
        
        for source, data in time_series_data.items():
            if len(data) < 10:
                continue
                
            try:
                # Find consolidation periods
                consolidations = self._find_consolidation_periods(data)
                
                for consolidation in consolidations:
                    # Check for breakouts
                    breakout = self._detect_breakout(data, consolidation)
                    
                    if breakout:
                        patterns.append(Pattern(
                            pattern_type='breakout',
                            confidence=breakout['confidence'],
                            description=f"Breakout detected: {breakout['direction']} from {breakout['range_size']:.2f} range",
                            data_source=source,
                            parameters=breakout
                        ))
                        
            except Exception as e:
                logger.error(f"Error detecting breakout patterns for {source}: {e}")
        
        return patterns
    
    async def _detect_support_resistance_patterns(self, time_series_data: Dict[str, List[float]]) -> List[Pattern]:
        """Detect support and resistance level patterns"""
        patterns = []
        
        for source, data in time_series_data.items():
            if len(data) < 10:
                continue
                
            try:
                # Find support and resistance levels
                levels = self._find_support_resistance_levels(data)
                
                # Analyze level strength and relevance
                for level in levels['support']:
                    if level['strength'] > 0.7:
                        patterns.append(Pattern(
                            pattern_type='support',
                            confidence=level['strength'],
                            description=f"Strong support level at {level['price']:.4f}",
                            data_source=source,
                            parameters=level
                        ))
                
                for level in levels['resistance']:
                    if level['strength'] > 0.7:
                        patterns.append(Pattern(
                            pattern_type='resistance',
                            confidence=level['strength'],
                            description=f"Strong resistance level at {level['price']:.4f}",
                            data_source=source,
                            parameters=level
                        ))
                        
            except Exception as e:
                logger.error(f"Error detecting support/resistance patterns for {source}: {e}")
        
        return patterns
    
    async def _detect_statistical_anomalies(self, time_series_data: Dict[str, List[float]]) -> List[Anomaly]:
        """Detect statistical anomalies using various methods"""
        anomalies = []
        
        for source, data in time_series_data.items():
            if len(data) < 5:
                continue
                
            try:
                # Z-score based anomaly detection
                z_anomalies = self._z_score_anomaly_detection(data)
                anomalies.extend([
                    Anomaly(
                        anomaly_type='statistical',
                        severity=self._classify_anomaly_severity(abs(z_score)),
                        description=f"Statistical outlier: z-score {z_score:.2f}",
                        value=value,
                        expected_value=np.mean(data),
                        deviation_score=abs(z_score),
                        timestamp=datetime.now()
                    )
                    for idx, value, z_score in z_anomalies
                ])
                
                # IQR based anomaly detection
                iqr_anomalies = self._iqr_anomaly_detection(data)
                anomalies.extend([
                    Anomaly(
                        anomaly_type='statistical',
                        severity='medium',
                        description=f"IQR outlier: value {value:.4f}",
                        value=value,
                        expected_value=np.median(data),
                        deviation_score=abs(value - np.median(data)),
                        timestamp=datetime.now()
                    )
                    for idx, value in iqr_anomalies
                ])
                
            except Exception as e:
                logger.error(f"Error detecting statistical anomalies for {source}: {e}")
        
        return anomalies
    
    async def _detect_behavioral_anomalies(self, time_series_data: Dict[str, List[float]]) -> List[Anomaly]:
        """Detect behavioral anomalies based on patterns and expectations"""
        anomalies = []
        
        for source, data in time_series_data.items():
            if len(data) < 10:
                continue
                
            try:
                # Sudden direction changes
                direction_changes = self._detect_sudden_direction_changes(data)
                for change in direction_changes:
                    anomalies.append(Anomaly(
                        anomaly_type='behavioral',
                        severity='medium',
                        description=f"Sudden direction change: {change['description']}",
                        value=change['value'],
                        expected_value=change['expected'],
                        deviation_score=change['magnitude'],
                        timestamp=datetime.now()
                    ))
                
                # Volume-price divergences (if volume data available)
                # This would require volume data in the time_series_data
                
                # Velocity anomalies
                velocity_anomalies = self._detect_velocity_anomalies(data)
                anomalies.extend(velocity_anomalies)
                
            except Exception as e:
                logger.error(f"Error detecting behavioral anomalies for {source}: {e}")
        
        return anomalies
    
    def _extract_time_series_data(self, data: Dict[str, Any]) -> Dict[str, List[float]]:
        """Extract time series data from various data formats"""
        time_series = {}
        
        # Try to extract time series from different data structures
        if isinstance(data, dict):
            for key, value in data.items():
                if isinstance(value, list) and len(value) > 0:
                    # Check if it's a list of numbers
                    try:
                        numeric_data = [float(x) for x in value if isinstance(x, (int, float))]
                        if len(numeric_data) > 2:
                            time_series[key] = numeric_data
                    except (ValueError, TypeError):
                        continue
                        
                # Handle nested data structures
                elif isinstance(value, dict):
                    nested_series = self._extract_time_series_data(value)
                    for nested_key, nested_data in nested_series.items():
                        time_series[f"{key}_{nested_key}"] = nested_data
        
        return time_series
    
    def _calculate_trend_metrics(self, data: List[float]) -> Tuple[float, str]:
        """Calculate trend strength and direction"""
        if len(data) < 2:
            return 0.0, "unknown"
        
        x = np.arange(len(data))
        y = np.array(data)
        
        # Linear regression
        correlation = np.corrcoef(x, y)[0, 1]
        trend_strength = abs(correlation)
        
        # Direction
        slope = np.polyfit(x, y, 1)[0]
        trend_direction = "up" if slope > 0 else "down"
        
        return trend_strength, trend_direction
    
    def _detect_trend_reversals(self, data: List[float]) -> List[Dict[str, Any]]:
        """Detect potential trend reversal points"""
        reversals = []
        
        if len(data) < 10:
            return reversals
        
        # Look for divergences in recent vs overall trend
        recent_trend_strength, recent_direction = self._calculate_trend_metrics(data[-5:])
        overall_trend_strength, overall_direction = self._calculate_trend_metrics(data)
        
        if (recent_trend_strength > 0.5 and overall_trend_strength > 0.5 and 
            recent_direction != overall_direction):
            reversals.append({
                'description': f"Recent trend {recent_direction} vs overall {overall_direction}",
                'confidence': min(recent_trend_strength, overall_trend_strength),
                'recent_strength': recent_trend_strength,
                'overall_strength': overall_trend_strength
            })
        
        # Look for momentum divergences
        momentum_signals = self._detect_momentum_divergence(data)
        reversals.extend(momentum_signals)
        
        return reversals
    
    def _find_cycles(self, data: List[float]) -> List[Dict[str, Any]]:
        """Find cyclical patterns in the data"""
        cycles = []
        
        if len(data) < 20:
            return cycles
        
        # Simple autocorrelation-based cycle detection
        for period in range(2, min(len(data) // 4, 20)):
            autocorr = self._calculate_autocorrelation(data, period)
            
            if autocorr > 0.6:  # Strong cyclical pattern
                # Calculate cycle characteristics
                amplitude = self._calculate_cycle_amplitude(data, period)
                phase = self._calculate_cycle_phase(data, period)
                
                cycles.append({
                    'period': period,
                    'confidence': autocorr,
                    'amplitude': amplitude,
                    'phase': phase
                })
        
        return sorted(cycles, key=lambda x: x['confidence'], reverse=True)
    
    def _detect_seasonal_patterns(self, data: List[float]) -> List[Pattern]:
        """Detect seasonal patterns (requires timestamp context)"""
        # This is a placeholder for seasonal pattern detection
        # Would require actual timestamp data to implement properly
        patterns = []
        
        # For now, just detect if data shows regular periodicity
        if len(data) >= 30:  # Assume monthly data
            monthly_pattern = self._test_periodicity(data, 30)
            if monthly_pattern['confidence'] > 0.7:
                patterns.append(Pattern(
                    pattern_type='seasonal',
                    confidence=monthly_pattern['confidence'],
                    description="Monthly seasonal pattern detected",
                    data_source='time_series',
                    parameters=monthly_pattern
                ))
        
        return patterns
    
    def _find_consolidation_periods(self, data: List[float]) -> List[Dict[str, Any]]:
        """Find periods of price consolidation (low volatility)"""
        consolidations = []
        
        if len(data) < 10:
            return consolidations
        
        # Use rolling window to find low volatility periods
        window_size = 5
        volatility_threshold = 0.02  # 2% volatility threshold
        
        for i in range(len(data) - window_size + 1):
            window = data[i:i + window_size]
            volatility = np.std(window) / np.mean(window)
            
            if volatility < volatility_threshold:
                consolidations.append({
                    'start_idx': i,
                    'end_idx': i + window_size - 1,
                    'volatility': volatility,
                    'price_range': max(window) - min(window),
                    'center_price': np.mean(window)
                })
        
        # Merge overlapping consolidation periods
        merged_consolidations = self._merge_consolidation_periods(consolidations)
        
        return merged_consolidations
    
    def _detect_breakout(self, data: List[float], consolidation: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Detect breakout from consolidation period"""
        end_idx = consolidation['end_idx']
        
        if end_idx >= len(data) - 1:
            return None
        
        # Check subsequent price movement
        breakout_window = data[end_idx + 1:end_idx + 4]  # Next 3 periods
        if not breakout_window:
            return None
        
        center_price = consolidation['center_price']
        price_range = consolidation['price_range']
        
        # Check for significant movement beyond consolidation range
        max_breakout = max(breakout_window)
        min_breakout = min(breakout_window)
        
        upward_breakout = max_breakout > center_price + price_range
        downward_breakout = min_breakout < center_price - price_range
        
        if upward_breakout or downward_breakout:
            direction = "upward" if upward_breakout else "downward"
            breakout_magnitude = (max_breakout - center_price) if upward_breakout else (center_price - min_breakout)
            
            return {
                'direction': direction,
                'magnitude': breakout_magnitude,
                'confidence': min(0.9, breakout_magnitude / price_range),
                'range_size': price_range,
                'consolidation_duration': consolidation['end_idx'] - consolidation['start_idx']
            }
        
        return None
    
    def _find_support_resistance_levels(self, data: List[float]) -> Dict[str, List[Dict[str, Any]]]:
        """Find support and resistance levels with strength analysis"""
        support_levels = []
        resistance_levels = []
        
        if len(data) < 5:
            return {'support': support_levels, 'resistance': resistance_levels}
        
        # Find local minima (potential support) and maxima (potential resistance)
        for i in range(2, len(data) - 2):
            current_price = data[i]
            
            # Check for local minimum (support)
            if (data[i] <= data[i-1] and data[i] <= data[i-2] and 
                data[i] <= data[i+1] and data[i] <= data[i+2]):
                
                # Calculate support strength
                strength = self._calculate_level_strength(data, i, current_price, 'support')
                support_levels.append({
                    'price': current_price,
                    'index': i,
                    'strength': strength,
                    'touches': self._count_level_touches(data, current_price, tolerance=0.01)
                })
            
            # Check for local maximum (resistance)
            if (data[i] >= data[i-1] and data[i] >= data[i-2] and 
                data[i] >= data[i+1] and data[i] >= data[i+2]):
                
                # Calculate resistance strength
                strength = self._calculate_level_strength(data, i, current_price, 'resistance')
                resistance_levels.append({
                    'price': current_price,
                    'index': i,
                    'strength': strength,
                    'touches': self._count_level_touches(data, current_price, tolerance=0.01)
                })
        
        # Sort by strength and return top levels
        support_levels.sort(key=lambda x: x['strength'], reverse=True)
        resistance_levels.sort(key=lambda x: x['strength'], reverse=True)
        
        return {
            'support': support_levels[:5],  # Top 5 support levels
            'resistance': resistance_levels[:5]  # Top 5 resistance levels
        }
    
    def _z_score_anomaly_detection(self, data: List[float], threshold: float = 2.5) -> List[Tuple[int, float, float]]:
        """Detect anomalies using z-score method"""
        if len(data) < 3:
            return []
        
        mean_val = np.mean(data)
        std_val = np.std(data)
        
        if std_val == 0:
            return []
        
        anomalies = []
        for i, value in enumerate(data):
            z_score = (value - mean_val) / std_val
            if abs(z_score) > threshold:
                anomalies.append((i, value, z_score))
        
        return anomalies
    
    def _iqr_anomaly_detection(self, data: List[float]) -> List[Tuple[int, float]]:
        """Detect anomalies using Interquartile Range method"""
        if len(data) < 4:
            return []
        
        q1 = np.percentile(data, 25)
        q3 = np.percentile(data, 75)
        iqr = q3 - q1
        
        lower_bound = q1 - 1.5 * iqr
        upper_bound = q3 + 1.5 * iqr
        
        anomalies = []
        for i, value in enumerate(data):
            if value < lower_bound or value > upper_bound:
                anomalies.append((i, value))
        
        return anomalies
    
    def _detect_sudden_direction_changes(self, data: List[float]) -> List[Dict[str, Any]]:
        """Detect sudden changes in price direction"""
        changes = []
        
        if len(data) < 5:
            return changes
        
        # Calculate rolling direction changes
        for i in range(2, len(data) - 2):
            before_trend = data[i] - data[i-2]
            after_trend = data[i+2] - data[i]
            
            # Check for significant direction reversal
            if before_trend * after_trend < 0 and abs(before_trend) > 0.01 and abs(after_trend) > 0.01:
                magnitude = abs(before_trend) + abs(after_trend)
                changes.append({
                    'description': f"Direction reversal at index {i}",
                    'value': data[i],
                    'expected': data[i-1],  # Simple expectation
                    'magnitude': magnitude,
                    'before_trend': before_trend,
                    'after_trend': after_trend
                })
        
        return changes
    
    def _detect_velocity_anomalies(self, data: List[float]) -> List[Anomaly]:
        """Detect anomalies in rate of change (velocity)"""
        anomalies = []
        
        if len(data) < 3:
            return anomalies
        
        # Calculate velocity (rate of change)
        velocities = [data[i] - data[i-1] for i in range(1, len(data))]
        
        # Find velocity anomalies
        velocity_anomalies = self._z_score_anomaly_detection(velocities, threshold=2.0)
        
        for idx, velocity, z_score in velocity_anomalies:
            anomalies.append(Anomaly(
                anomaly_type='velocity',
                severity=self._classify_anomaly_severity(abs(z_score)),
                description=f"Unusual rate of change: {velocity:.4f}",
                value=velocity,
                expected_value=np.mean(velocities),
                deviation_score=abs(z_score),
                timestamp=datetime.now()
            ))
        
        return anomalies
    
    def _classify_anomaly_severity(self, deviation_score: float) -> str:
        """Classify anomaly severity based on deviation score"""
        if deviation_score > 4.0:
            return 'high'
        elif deviation_score > 2.5:
            return 'medium'
        else:
            return 'low'
    
    def _calculate_autocorrelation(self, data: List[float], lag: int) -> float:
        """Calculate autocorrelation at given lag"""
        if len(data) <= lag:
            return 0.0
        
        x = np.array(data[:-lag])
        y = np.array(data[lag:])
        
        if len(x) == 0 or len(y) == 0:
            return 0.0
        
        correlation = np.corrcoef(x, y)[0, 1]
        return correlation if not np.isnan(correlation) else 0.0
    
    def _calculate_cycle_amplitude(self, data: List[float], period: int) -> float:
        """Calculate amplitude of cyclical pattern"""
        if len(data) < period * 2:
            return 0.0
        
        # Take full cycles
        num_cycles = len(data) // period
        cycle_maxes = []
        cycle_mins = []
        
        for i in range(num_cycles):
            cycle_data = data[i * period:(i + 1) * period]
            cycle_maxes.append(max(cycle_data))
            cycle_mins.append(min(cycle_data))
        
        avg_amplitude = np.mean([max_val - min_val for max_val, min_val in zip(cycle_maxes, cycle_mins)])
        return avg_amplitude
    
    def _calculate_cycle_phase(self, data: List[float], period: int) -> float:
        """Calculate phase of cyclical pattern"""
        # Simplified phase calculation
        # In practice, this would use more sophisticated signal processing
        if len(data) < period:
            return 0.0
        
        first_cycle = data[:period]
        max_idx = first_cycle.index(max(first_cycle))
        phase = max_idx / period
        
        return phase
    
    def _test_periodicity(self, data: List[float], period: int) -> Dict[str, Any]:
        """Test for periodicity at given period"""
        if len(data) < period * 2:
            return {'confidence': 0.0}
        
        # Compare cycles
        num_cycles = len(data) // period
        cycle_correlations = []
        
        for i in range(num_cycles - 1):
            cycle1 = data[i * period:(i + 1) * period]
            cycle2 = data[(i + 1) * period:(i + 2) * period]
            
            if len(cycle1) == len(cycle2):
                corr = np.corrcoef(cycle1, cycle2)[0, 1]
                if not np.isnan(corr):
                    cycle_correlations.append(corr)
        
        avg_correlation = np.mean(cycle_correlations) if cycle_correlations else 0.0
        
        return {
            'confidence': max(0.0, avg_correlation),
            'period': period,
            'num_cycles': num_cycles,
            'correlations': cycle_correlations
        }
    
    def _merge_consolidation_periods(self, consolidations: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Merge overlapping consolidation periods"""
        if not consolidations:
            return []
        
        # Sort by start index
        consolidations.sort(key=lambda x: x['start_idx'])
        
        merged = []
        current = consolidations[0].copy()
        
        for next_consolidation in consolidations[1:]:
            if next_consolidation['start_idx'] <= current['end_idx']:
                # Overlapping - merge
                current['end_idx'] = max(current['end_idx'], next_consolidation['end_idx'])
                current['volatility'] = min(current['volatility'], next_consolidation['volatility'])
            else:
                # No overlap - add current and start new
                merged.append(current)
                current = next_consolidation.copy()
        
        merged.append(current)
        return merged
    
    def _calculate_level_strength(self, data: List[float], level_idx: int, level_price: float, level_type: str) -> float:
        """Calculate strength of support/resistance level"""
        # Factors: number of touches, volume at level, time since level, etc.
        
        # Count nearby touches
        touches = self._count_level_touches(data, level_price, tolerance=0.01)
        
        # Recent relevance (closer to end = higher strength)
        recency_factor = 1.0 - (len(data) - level_idx) / len(data)
        
        # Base strength calculation
        strength = min(1.0, (touches * 0.2) + (recency_factor * 0.5))
        
        return strength
    
    def _count_level_touches(self, data: List[float], level_price: float, tolerance: float = 0.01) -> int:
        """Count how many times price touched a level"""
        touches = 0
        tolerance_range = level_price * tolerance
        
        for price in data:
            if abs(price - level_price) <= tolerance_range:
                touches += 1
        
        return touches
    
    def _detect_momentum_divergence(self, data: List[float]) -> List[Dict[str, Any]]:
        """Detect momentum divergences"""
        divergences = []
        
        if len(data) < 10:
            return divergences
        
        # Simple momentum calculation (rate of change)
        momentum = [data[i] - data[i-1] for i in range(1, len(data))]
        
        # Look for price vs momentum divergences
        recent_price_trend = data[-1] - data[-5] if len(data) >= 5 else 0
        recent_momentum_trend = sum(momentum[-4:]) if len(momentum) >= 4 else 0
        
        # Bearish divergence: price up, momentum down
        if recent_price_trend > 0 and recent_momentum_trend < 0:
            divergences.append({
                'description': 'Bearish momentum divergence',
                'confidence': 0.7,
                'price_trend': recent_price_trend,
                'momentum_trend': recent_momentum_trend
            })
        
        # Bullish divergence: price down, momentum up
        elif recent_price_trend < 0 and recent_momentum_trend > 0:
            divergences.append({
                'description': 'Bullish momentum divergence',
                'confidence': 0.7,
                'price_trend': recent_price_trend,
                'momentum_trend': recent_momentum_trend
            })
        
        return divergences
    
    def _store_patterns(self, data_source: str, patterns: List[Pattern]):
        """Store detected patterns for historical analysis"""
        if data_source not in self.pattern_history:
            self.pattern_history[data_source] = []
        
        self.pattern_history[data_source].extend(patterns)
        
        # Keep only recent patterns (last 100)
        self.pattern_history[data_source] = self.pattern_history[data_source][-100:]
    
    def _store_anomalies(self, data_source: str, anomalies: List[Anomaly]):
        """Store detected anomalies for historical analysis"""
        if data_source not in self.anomaly_history:
            self.anomaly_history[data_source] = []
        
        self.anomaly_history[data_source].extend(anomalies)
        
        # Keep only recent anomalies (last 50)
        self.anomaly_history[data_source] = self.anomaly_history[data_source][-50:]
    
    def _generate_pattern_summary(self, patterns: List[Pattern]) -> Dict[str, Any]:
        """Generate summary of detected patterns"""
        if not patterns:
            return {'total_patterns': 0, 'pattern_types': {}}
        
        pattern_types = {}
        total_confidence = 0.0
        
        for pattern in patterns:
            pattern_type = pattern.pattern_type
            if pattern_type not in pattern_types:
                pattern_types[pattern_type] = 0
            pattern_types[pattern_type] += 1
            total_confidence += pattern.confidence
        
        return {
            'total_patterns': len(patterns),
            'pattern_types': pattern_types,
            'average_confidence': total_confidence / len(patterns),
            'strongest_pattern': max(patterns, key=lambda p: p.confidence).pattern_type
        }
    
    def _generate_anomaly_summary(self, anomalies: List[Anomaly]) -> Dict[str, Any]:
        """Generate summary of detected anomalies"""
        if not anomalies:
            return {'total_anomalies': 0, 'severity_distribution': {}}
        
        severity_distribution = {'low': 0, 'medium': 0, 'high': 0}
        anomaly_types = {}
        
        for anomaly in anomalies:
            severity_distribution[anomaly.severity] += 1
            
            anomaly_type = anomaly.anomaly_type
            if anomaly_type not in anomaly_types:
                anomaly_types[anomaly_type] = 0
            anomaly_types[anomaly_type] += 1
        
        return {
            'total_anomalies': len(anomalies),
            'severity_distribution': severity_distribution,
            'anomaly_types': anomaly_types,
            'high_severity_count': severity_distribution['high']
        }
    
    async def _update_baseline_metrics(self):
        """Update baseline metrics for anomaly detection"""
        # Update baseline metrics based on recent patterns and anomalies
        for data_source in self.pattern_history:
            recent_patterns = self.pattern_history[data_source][-20:]  # Last 20 patterns
            
            if recent_patterns:
                # Calculate baseline pattern characteristics
                avg_confidence = np.mean([p.confidence for p in recent_patterns])
                pattern_frequency = len(recent_patterns)
                
                self.baseline_metrics[data_source] = {
                    'avg_pattern_confidence': avg_confidence,
                    'pattern_frequency': pattern_frequency,
                    'last_updated': datetime.now().timestamp()
                }
    
    async def _cleanup_old_data(self):
        """Clean up old patterns and anomalies"""
        cutoff_time = datetime.now() - timedelta(hours=24)
        
        # Clean old anomalies
        for data_source in list(self.anomaly_history.keys()):
            self.anomaly_history[data_source] = [
                anomaly for anomaly in self.anomaly_history[data_source]
                if anomaly.timestamp > cutoff_time
            ]
            
            # Remove empty entries
            if not self.anomaly_history[data_source]:
                del self.anomaly_history[data_source]