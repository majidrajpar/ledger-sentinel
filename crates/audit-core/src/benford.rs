use serde::{Deserialize, Serialize};
use crate::models::{AuditFinding, RiskSeverity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitStats {
    pub digit: u32,
    pub observed_count: usize,
    pub actual_proportion: f64,
    pub expected_proportion: f64,
    pub z_score: f64,
    pub is_anomalous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenfordReport {
    pub is_eligible: bool,
    pub ineligibility_reason: Option<String>,
    pub population_size: usize,
    pub min_value: f64,
    pub max_value: f64,
    pub mad: f64,
    pub conformity: String,
    pub first_digits: Vec<DigitStats>,
    pub second_digits: Vec<DigitStats>,
}

pub fn analyze_benford(amounts: &[f64]) -> (BenfordReport, Vec<AuditFinding>) {
    let mut findings = Vec::new();

    // Filter positive amounts >= 1.0 (exclude zero, negatives, and small cents)
    let valid: Vec<f64> = amounts.iter()
        .copied()
        .filter(|&v| v.is_finite() && v >= 1.0)
        .collect();

    let n = valid.len();

    // Check minimum population
    if n < 100 {
        let report = BenfordReport {
            is_eligible: false,
            ineligibility_reason: Some(format!("Sample size too small (N = {}, minimum required = 100)", n)),
            population_size: n,
            min_value: valid.first().copied().unwrap_or(0.0),
            max_value: valid.last().copied().unwrap_or(0.0),
            mad: 0.0,
            conformity: "Ineligible".to_string(),
            first_digits: Vec::new(),
            second_digits: Vec::new(),
        };
        return (report, findings);
    }

    let mut min_val = f64::MAX;
    let mut max_val = f64::MIN;
    for &v in &valid {
        if v < min_val { min_val = v; }
        if v > max_val { max_val = v; }
    }

    // Check magnitude span: Nigrini requires spanning several orders of magnitude
    let ratio = if min_val > 0.0 { max_val / min_val } else { 1.0 };
    if ratio < 10.0 {
        let report = BenfordReport {
            is_eligible: false,
            ineligibility_reason: Some(format!(
                "Constrained price distribution (max/min ratio = {:.1} < 10). Benford analysis is invalid for fixed-rate populations.",
                ratio
            )),
            population_size: n,
            min_value: min_val,
            max_value: max_val,
            mad: 0.0,
            conformity: "Ineligible (Constrained Domain)".to_string(),
            first_digits: Vec::new(),
            second_digits: Vec::new(),
        };
        return (report, findings);
    }

    // 1st Digit Analysis (1..=9)
    let mut first_counts = vec![0usize; 10]; // index 1..=9
    let mut second_counts = vec![0usize; 10]; // index 0..=9

    for &v in &valid {
        let s = v.to_string();
        let digits: Vec<u32> = s.chars()
            .filter(|c| c.is_ascii_digit())
            .filter_map(|c| c.to_digit(10))
            .collect();

        if let Some(&d1) = digits.first() {
            if d1 >= 1 && d1 <= 9 {
                first_counts[d1 as usize] += 1;
            }
        }

        if let Some(&d2) = digits.get(1) {
            if d2 <= 9 {
                second_counts[d2 as usize] += 1;
            }
        }
    }

    let n_f64 = n as f64;
    let mut total_abs_dev = 0.0;
    let mut first_stats = Vec::new();

    for d in 1..=9 {
        let expected = (1.0 + 1.0 / (d as f64)).log10();
        let actual = first_counts[d] as f64 / n_f64;
        let abs_dev = (actual - expected).abs();
        total_abs_dev += abs_dev;

        // Continuity corrected Z-score
        let var = (expected * (1.0 - expected)) / n_f64;
        let z = if var > 0.0 && abs_dev > (1.0 / (2.0 * n_f64)) {
            (abs_dev - (1.0 / (2.0 * n_f64))) / var.sqrt()
        } else {
            0.0
        };

        let is_anomalous = z >= 3.29; // 99.9% confidence threshold
        if is_anomalous && actual > expected {
            findings.push(AuditFinding {
                rule: "BENFORD_DIGIT_SPIKE".to_string(),
                severity: if z >= 4.5 { RiskSeverity::High } else { RiskSeverity::Medium },
                title: format!("Anomalous Spike at First Digit '{}'", d),
                description: format!(
                    "Digit '{}' appears in {:.2}% of entries (expected {:.2}%, z-score: {:.2}). Indicates potential artificial estimation or automated rounding.",
                    d, actual * 100.0, expected * 100.0, z
                ),
                transaction_id: None,
                exposure_amount: first_counts[d] as f64 * (max_val + min_val) / 4.0,
                raw_line: None,
                metadata: serde_json::json!({
                    "digit": d,
                    "actual_pct": actual * 100.0,
                    "expected_pct": expected * 100.0,
                    "z_score": z,
                    "count": first_counts[d]
                }),
            });
        }

        first_stats.push(DigitStats {
            digit: d as u32,
            observed_count: first_counts[d],
            actual_proportion: actual,
            expected_proportion: expected,
            z_score: z,
            is_anomalous,
        });
    }

    // Mean Absolute Deviation
    let mad = total_abs_dev / 9.0;
    let conformity = if mad <= 0.006 {
        "Close conformity"
    } else if mad <= 0.012 {
        "Acceptable conformity"
    } else if mad <= 0.015 {
        "Marginally acceptable conformity"
    } else {
        "Nonconforming"
    };

    if conformity == "Nonconforming" {
        findings.push(AuditFinding {
            rule: "BENFORD_POPULATION_NONCONFORMITY".to_string(),
            severity: RiskSeverity::Medium,
            title: "Ledger Population Nonconformity with Benford's Law".to_string(),
            description: format!(
                "The entire population has a Mean Absolute Deviation (MAD) of {:.4}, exceeding the Nigrini threshold of 0.015. Suggests non-natural disbursement patterns.",
                mad
            ),
            transaction_id: None,
            exposure_amount: 0.0,
            raw_line: None,
            metadata: serde_json::json!({
                "mad": mad,
                "conformity": conformity,
                "population_size": n
            }),
        });
    }

    // 2nd Digit Analysis (0..=9)
    let mut second_stats = Vec::new();
    for d in 0..=9 {
        // Expected 2nd digit proportion: sum_{p=1}^9 log10(1 + 1/(10p + d))
        let expected: f64 = (1..=9)
            .map(|p| (1.0 + 1.0 / (10.0 * (p as f64) + (d as f64))).log10())
            .sum();

        let actual = second_counts[d] as f64 / n_f64;
        let abs_dev = (actual - expected).abs();
        let var = (expected * (1.0 - expected)) / n_f64;
        let z = if var > 0.0 && abs_dev > (1.0 / (2.0 * n_f64)) {
            (abs_dev - (1.0 / (2.0 * n_f64))) / var.sqrt()
        } else {
            0.0
        };

        second_stats.push(DigitStats {
            digit: d as u32,
            observed_count: second_counts[d],
            actual_proportion: actual,
            expected_proportion: expected,
            z_score: z,
            is_anomalous: z >= 3.29,
        });
    }

    let report = BenfordReport {
        is_eligible: true,
        ineligibility_reason: None,
        population_size: n,
        min_value: min_val,
        max_value: max_val,
        mad,
        conformity: conformity.to_string(),
        first_digits: first_stats,
        second_digits: second_stats,
    };

    (report, findings)
}
