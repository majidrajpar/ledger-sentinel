pub mod models;
pub mod schema;
pub mod benford;
pub mod structuring;
pub mod sod;
pub mod duplicates;
pub mod round_sums;

use std::collections::HashMap;
use models::{AuditReport, RiskSeverity, RuleSummary, SeveritySummary, Transaction};
use structuring::StructuringConfig;

pub fn run_full_audit(
    transactions: &[Transaction],
    structuring_cfg: Option<StructuringConfig>,
) -> AuditReport {
    let mut findings = Vec::new();

    // 1. Benford's Law Analysis on amounts
    let amounts: Vec<f64> = transactions.iter().map(|t| t.amount).collect();
    let (_benford_rep, benford_findings) = benford::analyze_benford(&amounts);
    findings.extend(benford_findings);

    // 2. Structuring / Split Purchase Orders
    let cfg = structuring_cfg.unwrap_or_default();
    let struct_findings = structuring::detect_structuring(transactions, &cfg);
    findings.extend(struct_findings);

    // 3. Segregation of Duties (Maker-Checker Collisions)
    let sod_findings = sod::detect_sod_breaches(transactions);
    findings.extend(sod_findings);

    // 4. Fuzzy Duplicate Payments
    let dup_findings = duplicates::detect_fuzzy_duplicates(transactions);
    findings.extend(dup_findings);

    // 5. Round Sum Overrides
    let round_findings = round_sums::detect_round_sums(transactions);
    findings.extend(round_findings);

    // Sort findings by severity (Critical -> High -> Medium -> Low) then exposure amount descending
    findings.sort_by(|a, b| {
        b.severity.cmp(&a.severity)
            .then_with(|| b.exposure_amount.partial_cmp(&a.exposure_amount).unwrap_or(std::cmp::Ordering::Equal))
    });

    let total_exposure: f64 = findings.iter().map(|f| f.exposure_amount).sum();

    // Aggregate summary statistics
    let mut rule_map: HashMap<String, (usize, f64, RiskSeverity)> = HashMap::new();
    let mut severity_summary = SeveritySummary::default();

    for f in &findings {
        match f.severity {
            RiskSeverity::Critical => severity_summary.critical_count += 1,
            RiskSeverity::High => severity_summary.high_count += 1,
            RiskSeverity::Medium => severity_summary.medium_count += 1,
            RiskSeverity::Low => severity_summary.low_count += 1,
        }

        let entry = rule_map.entry(f.rule.clone()).or_insert((0, 0.0, f.severity));
        entry.0 += 1;
        entry.1 += f.exposure_amount;
        if f.severity > entry.2 {
            entry.2 = f.severity;
        }
    }

    let mut summary_by_rule: Vec<RuleSummary> = rule_map.into_iter().map(|(rule, (count, exp, sev))| {
        RuleSummary {
            rule,
            count,
            total_exposure: exp,
            highest_severity: sev,
        }
    }).collect();

    summary_by_rule.sort_by(|a, b| b.total_exposure.partial_cmp(&a.total_exposure).unwrap_or(std::cmp::Ordering::Equal));

    AuditReport {
        total_rows_scanned: transactions.len(),
        total_exposure_at_risk: total_exposure,
        findings,
        summary_by_rule,
        summary_by_severity: severity_summary,
    }
}
