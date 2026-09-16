use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use audit_core::models::{AuditReport, RiskSeverity};

pub fn print_report_table(report: &AuditReport, max_findings: usize) {
    println!("\n================================================================================");
    println!("                        LEDGER-SENTINEL CONTINUOUS CONTROLS REPORT              ");
    println!("================================================================================\n");

    // 1. Executive Summary Table
    let mut exec_table = Table::new();
    exec_table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Total Rows Scanned",
            "Total Anomalies",
            "Critical",
            "High",
            "Medium",
            "Low",
            "Total Exposure at Risk",
        ]);

    exec_table.add_row(vec![
        Cell::new(format!("{}", report.total_rows_scanned)),
        Cell::new(format!("{}", report.findings.len())),
        Cell::new(format!("{}", report.summary_by_severity.critical_count)).fg(Color::Red),
        Cell::new(format!("{}", report.summary_by_severity.high_count)).fg(Color::DarkRed),
        Cell::new(format!("{}", report.summary_by_severity.medium_count)).fg(Color::Yellow),
        Cell::new(format!("{}", report.summary_by_severity.low_count)).fg(Color::Blue),
        Cell::new(format!("${:.2}", report.total_exposure_at_risk)).fg(Color::Green),
    ]);

    println!("--- EXECUTIVE RISK SUMMARY ---");
    println!("{exec_table}\n");

    // 2. Rule Breakdown Table
    if !report.summary_by_rule.is_empty() {
        let mut rule_table = Table::new();
        rule_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Control / Forensic Rule", "Findings", "Highest Severity", "Financial Exposure"]);

        for s in &report.summary_by_rule {
            let sev_color = match s.highest_severity {
                RiskSeverity::Critical => Color::Red,
                RiskSeverity::High => Color::DarkRed,
                RiskSeverity::Medium => Color::Yellow,
                RiskSeverity::Low => Color::Blue,
            };

            rule_table.add_row(vec![
                Cell::new(&s.rule),
                Cell::new(format!("{}", s.count)),
                Cell::new(format!("{}", s.highest_severity)).fg(sev_color),
                Cell::new(format!("${:.2}", s.total_exposure)),
            ]);
        }

        println!("--- FINDINGS BY CONTROL VECTOR ---");
        println!("{rule_table}\n");
    }

    // 3. Top Detailed Findings Table
    if !report.findings.is_empty() {
        let display_count = report.findings.len().min(max_findings);
        println!("--- TOP {} ANOMALIES (RANKED BY SEVERITY & EXPOSURE) ---", display_count);

        let mut detail_table = Table::new();
        detail_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Severity", "Rule", "Ref / Tx ID", "Line", "Exposure", "Details"]);

        for f in report.findings.iter().take(display_count) {
            let sev_color = match f.severity {
                RiskSeverity::Critical => Color::Red,
                RiskSeverity::High => Color::DarkRed,
                RiskSeverity::Medium => Color::Yellow,
                RiskSeverity::Low => Color::Blue,
            };

            let tx_str = f.transaction_id.as_deref().unwrap_or("-");
            let line_str = f.raw_line.map(|l| l.to_string()).unwrap_or_else(|| "-".to_string());

            detail_table.add_row(vec![
                Cell::new(format!("{}", f.severity)).fg(sev_color),
                Cell::new(&f.rule),
                Cell::new(tx_str),
                Cell::new(line_str),
                Cell::new(format!("${:.2}", f.exposure_amount)),
                Cell::new(&f.description),
            ]);
        }

        println!("{detail_table}\n");
    }
}
