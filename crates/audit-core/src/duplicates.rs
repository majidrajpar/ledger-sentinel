use std::collections::HashMap;
use crate::models::{AuditFinding, RiskSeverity, Transaction};

fn normalize_ref(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_uppercase()
        .trim_start_matches('0')
        .to_string()
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[m][n]
}

pub fn detect_fuzzy_duplicates(transactions: &[Transaction]) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    // Group by (Vendor, Rounded Amount in Cents)
    let mut groups: HashMap<(String, i64), Vec<&Transaction>> = HashMap::new();

    for tx in transactions {
        if let Some(v) = &tx.vendor {
            let cents = (tx.amount * 100.0).round() as i64;
            if cents > 0 {
                groups.entry((v.trim().to_uppercase(), cents)).or_default().push(tx);
            }
        }
    }

    for ((vendor, cents), items) in groups {
        if items.len() < 2 { continue; }

        let mut flagged_pairs = Vec::new();
        for i in 0..items.len() {
            for j in (i + 1)..items.len() {
                let t1 = items[i];
                let t2 = items[j];

                let norm1 = normalize_ref(&t1.id);
                let norm2 = normalize_ref(&t2.id);

                let dist = levenshtein(&norm1, &norm2);
                // Exact match on normalized, or edit distance of 1 (transposed/miskeyed character)
                if norm1 == norm2 || (dist <= 1 && norm1.len() >= 4) {
                    flagged_pairs.push((t1, t2, norm1, norm2, dist));
                }
            }
        }

        for (t1, t2, norm1, _norm2, dist) in flagged_pairs {
            let amount = cents as f64 / 100.0;
            let severity = if amount >= 10000.0 {
                RiskSeverity::Critical
            } else if amount >= 1000.0 {
                RiskSeverity::High
            } else {
                RiskSeverity::Medium
            };

            let reason = if dist == 0 {
                format!("Exact normalized invoice collision ('{}')", norm1)
            } else {
                format!("Single character variation ('{}' vs '{}')", t1.id, t2.id)
            };

            findings.push(AuditFinding {
                rule: "POTENTIAL_DUPLICATE_PAYMENT".to_string(),
                severity,
                title: format!("Duplicate Payment to Vendor '{}' (${:.2})", vendor, amount),
                description: format!(
                    "Duplicate disbursements detected: Ref '{}' (line {}) and Ref '{}' (line {}) for ${:.2}. Reason: {}.",
                    t1.id, t1.raw_line, t2.id, t2.raw_line, amount, reason
                ),
                transaction_id: Some(format!("{}, {}", t1.id, t2.id)),
                exposure_amount: amount,
                raw_line: Some(t1.raw_line),
                metadata: serde_json::json!({
                    "vendor": vendor,
                    "amount": amount,
                    "first_id": t1.id,
                    "second_id": t2.id,
                    "first_line": t1.raw_line,
                    "second_line": t2.raw_line,
                    "distance": dist
                }),
            });
        }
    }

    findings
}
