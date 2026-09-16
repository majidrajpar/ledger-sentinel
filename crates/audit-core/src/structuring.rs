use std::collections::HashMap;
use crate::models::{AuditFinding, RiskSeverity, Transaction};

#[derive(Debug, Clone)]
pub struct StructuringConfig {
    pub threshold: f64,
    pub margin_pct: f64,        // e.g. 0.10 means 90% to 99.9% of threshold
    pub min_repetitions: usize, // e.g. 2 or more clustered transactions
}

impl Default for StructuringConfig {
    fn default() -> Self {
        Self {
            threshold: 5000.0,
            margin_pct: 0.10,
            min_repetitions: 2,
        }
    }
}

pub fn detect_structuring(
    transactions: &[Transaction],
    config: &StructuringConfig,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    let lower_bound = config.threshold * (1.0 - config.margin_pct);
    let upper_bound = config.threshold;

    // Group candidates by vendor or account
    let mut vendor_candidates: HashMap<String, Vec<&Transaction>> = HashMap::new();

    for tx in transactions {
        if tx.amount >= lower_bound && tx.amount < upper_bound {
            let key = tx.vendor.clone().unwrap_or_else(|| "UNKNOWN_VENDOR".to_string());
            vendor_candidates.entry(key).or_default().push(tx);
        }
    }

    for (vendor, items) in vendor_candidates {
        if items.len() >= config.min_repetitions {
            let total_exposure: f64 = items.iter().map(|t| t.amount).sum();
            let tx_ids: Vec<String> = items.iter().map(|t| t.id.clone()).collect();
            let lines: Vec<usize> = items.iter().map(|t| t.raw_line).collect();

            let severity = if items.len() >= 4 || total_exposure >= config.threshold * 3.0 {
                RiskSeverity::Critical
            } else if items.len() >= 3 || total_exposure >= config.threshold * 2.0 {
                RiskSeverity::High
            } else {
                RiskSeverity::Medium
            };

            findings.push(AuditFinding {
                rule: "SPLIT_APPROVAL_STRUCTURING".to_string(),
                severity,
                title: format!("Split Purchases Below ${:.0} Limit ({})", config.threshold, vendor),
                description: format!(
                    "{} transactions totaling ${:.2} clustered in the ${:.0}-${:.0} bracket (90-100% of delegation limit) for vendor '{}'. Indicates circumvention of authority limits.",
                    items.len(), total_exposure, lower_bound, upper_bound, vendor
                ),
                transaction_id: Some(tx_ids.join(", ")),
                exposure_amount: total_exposure,
                raw_line: lines.first().copied(),
                metadata: serde_json::json!({
                    "vendor": vendor,
                    "count": items.len(),
                    "threshold": config.threshold,
                    "total_amount": total_exposure,
                    "transaction_ids": tx_ids,
                    "raw_lines": lines
                }),
            });
        }
    }

    findings
}
