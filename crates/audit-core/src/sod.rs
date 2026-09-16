use crate::models::{AuditFinding, RiskSeverity, Transaction};

const SYSTEM_ACCOUNT_PREFIXES: &[&str] = &[
    "SYSTEM", "SYS", "AUTO", "CRON", "BATCH", "INTERFACE",
    "INTEGRATION", "MIGRATION", "DAEMON", "SVC", "BOT", "SCHEDULED"
];

fn is_system_account(name: &str) -> bool {
    let upper = name.trim().to_uppercase();
    SYSTEM_ACCOUNT_PREFIXES.iter().any(|&p| upper.contains(p))
}

pub fn detect_sod_breaches(transactions: &[Transaction]) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    for tx in transactions {
        if let (Some(maker), Some(checker)) = (&tx.created_by, &tx.approved_by) {
            let clean_maker = maker.trim();
            let clean_checker = checker.trim();

            if !clean_maker.is_empty() 
                && !clean_checker.is_empty() 
                && clean_maker.eq_ignore_ascii_case(clean_checker) 
            {
                // Filter out automated system accounts
                if is_system_account(clean_maker) {
                    continue;
                }

                let severity = if tx.amount >= 10000.0 {
                    RiskSeverity::Critical
                } else if tx.amount >= 2500.0 {
                    RiskSeverity::High
                } else {
                    RiskSeverity::Medium
                };

                findings.push(AuditFinding {
                    rule: "MAKER_CHECKER_SOD_COLLISION".to_string(),
                    severity,
                    title: format!("Self-Approved Entry by '{}'", clean_maker),
                    description: format!(
                        "Transaction '{}' for ${:.2} was created and approved by the same user ID ('{}'). Segregation of duties control bypassed.",
                        tx.id, tx.amount, clean_maker
                    ),
                    transaction_id: Some(tx.id.clone()),
                    exposure_amount: tx.amount,
                    raw_line: Some(tx.raw_line),
                    metadata: serde_json::json!({
                        "user_id": clean_maker,
                        "transaction_id": tx.id,
                        "amount": tx.amount,
                        "account": tx.account,
                        "vendor": tx.vendor
                    }),
                });
            }
        }
    }

    findings
}
