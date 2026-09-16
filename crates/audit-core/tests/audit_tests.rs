use audit_core::models::{RiskSeverity, Transaction};
use audit_core::structuring::StructuringConfig;
use audit_core::run_full_audit;

fn sample_tx(id: &str, amount: f64, vendor: &str, maker: &str, checker: &str) -> Transaction {
    Transaction {
        id: id.to_string(),
        amount,
        date: Some("2026-03-15".to_string()),
        vendor: Some(vendor.to_string()),
        account: Some("6010-Consulting".to_string()),
        created_by: Some(maker.to_string()),
        approved_by: Some(checker.to_string()),
        entry_type: Some("AP_INVOICE".to_string()),
        description: Some(format!("Payment {}", id)),
        raw_line: 1,
    }
}

#[test]
fn test_sod_collision_detection() {
    let txs = vec![
        sample_tx("TX-001", 12500.0, "Vendor A", "john.doe", "john.doe"),
        sample_tx("TX-002", 500.0, "Vendor B", "alice.smith", "bob.jones"),
    ];

    let report = run_full_audit(&txs, None);
    let sod_findings: Vec<_> = report.findings.iter().filter(|f| f.rule == "MAKER_CHECKER_SOD_COLLISION").collect();
    
    assert_eq!(sod_findings.len(), 1);
    assert_eq!(sod_findings[0].severity, RiskSeverity::Critical);
    assert_eq!(sod_findings[0].exposure_amount, 12500.0);
}

#[test]
fn test_structuring_split_purchases() {
    let cfg = StructuringConfig {
        threshold: 5000.0,
        margin_pct: 0.10,
        min_repetitions: 2,
    };

    let txs = vec![
        sample_tx("APX-01", 4950.0, "Apex Consulting", "alice", "bob"),
        sample_tx("APX-02", 4920.0, "Apex Consulting", "alice", "bob"),
        sample_tx("ORD-01", 1200.0, "Other Vendor", "alice", "bob"),
    ];

    let report = run_full_audit(&txs, Some(cfg));
    let struct_findings: Vec<_> = report.findings.iter().filter(|f| f.rule == "SPLIT_APPROVAL_STRUCTURING").collect();

    assert_eq!(struct_findings.len(), 1);
    assert_eq!(struct_findings[0].exposure_amount, 4950.0 + 4920.0);
}

#[test]
fn test_fuzzy_duplicate_payments() {
    let txs = vec![
        sample_tx("INV-9042", 8750.50, "Global Logistics", "alice", "bob"),
        sample_tx("INV9042", 8750.50, "Global Logistics", "alice", "bob"),
        sample_tx("INV-9043", 200.0, "Global Logistics", "alice", "bob"),
    ];

    let report = run_full_audit(&txs, None);
    let dup_findings: Vec<_> = report.findings.iter().filter(|f| f.rule == "POTENTIAL_DUPLICATE_PAYMENT").collect();

    assert_eq!(dup_findings.len(), 1);
    assert_eq!(dup_findings[0].exposure_amount, 8750.50);
}

#[test]
fn test_round_sum_manual_override() {
    let txs = vec![
        sample_tx("JV-50000", 50000.0, "Delta Corp", "alice", "bob"),
        sample_tx("JV-50001", 1234.56, "Delta Corp", "alice", "bob"),
    ];

    let report = run_full_audit(&txs, None);
    let round_findings: Vec<_> = report.findings.iter().filter(|f| f.rule == "ROUND_SUM_OVERRIDE").collect();

    assert_eq!(round_findings.len(), 1);
    assert_eq!(round_findings[0].exposure_amount, 50000.0);
}
