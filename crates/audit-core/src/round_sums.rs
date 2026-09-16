use crate::models::{AuditFinding, RiskSeverity, Transaction};

pub fn detect_round_sums(transactions: &[Transaction]) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    for tx in transactions {
        if tx.amount >= 10000.0 {
            let cents = (tx.amount * 100.0).round() as i64;
            // Check if exact multiple of 1,000 (100,000 cents) or 5,000
            if cents % 100000 == 0 {
                let is_large_round = tx.amount >= 25000.0;
                let severity = if tx.amount >= 50000.0 {
                    RiskSeverity::High
                } else if is_large_round {
                    RiskSeverity::Medium
                } else {
                    RiskSeverity::Low
                };

                let entry_desc = tx.description.as_deref().unwrap_or("No narration provided");

                findings.push(AuditFinding {
                    rule: "ROUND_SUM_OVERRIDE".to_string(),
                    severity,
                    title: format!("Round Thousand Transaction (${:.0})", tx.amount),
                    description: format!(
                        "Transaction '{}' (line {}) has an exact round-thousand amount of ${:.2}. Narration: '{}'. Common indicator of manual management override or unsupported accrual.",
                        tx.id, tx.raw_line, tx.amount, entry_desc
                    ),
                    transaction_id: Some(tx.id.clone()),
                    exposure_amount: tx.amount,
                    raw_line: Some(tx.raw_line),
                    metadata: serde_json::json!({
                        "amount": tx.amount,
                        "account": tx.account,
                        "vendor": tx.vendor,
                        "maker": tx.created_by,
                        "description": entry_desc
                    }),
                });
            }
        }
    }

    findings
}
