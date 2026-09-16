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

pub fn print_benford_table(report: &audit_core::benford::BenfordReport) {
    println!("\n================================================================================");
    println!("                           BENFORD'S LAW EVALUATION                             ");
    println!("================================================================================\n");

    let mut meta_table = Table::new();
    meta_table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Eligibility",
            "Population (N)",
            "Min Amount",
            "Max Amount",
            "Mean Abs Dev (MAD)",
            "Conformity Classification",
        ]);

    let elig_color = if report.is_eligible { Color::Green } else { Color::Red };
    let conf_color = match report.conformity.as_str() {
        "Close conformity" => Color::Green,
        "Acceptable conformity" => Color::Cyan,
        "Marginally acceptable conformity" => Color::Yellow,
        _ => Color::Red,
    };

    meta_table.add_row(vec![
        Cell::new(if report.is_eligible { "ELIGIBLE" } else { "INELIGIBLE" }).fg(elig_color),
        Cell::new(format!("{}", report.population_size)),
        Cell::new(format!("${:.2}", report.min_value)),
        Cell::new(format!("${:.2}", report.max_value)),
        Cell::new(format!("{:.4}", report.mad)),
        Cell::new(&report.conformity).fg(conf_color),
    ]);

    println!("{meta_table}\n");

    if let Some(reason) = &report.ineligibility_reason {
        println!("Eligibility Note: {reason}\n");
    }

    if !report.first_digits.is_empty() {
        println!("--- FIRST DIGIT FREQUENCY & Z-SCORE ANOMALIES ---");
        let mut digit_table = Table::new();
        digit_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                "Digit",
                "Observed",
                "Actual %",
                "Expected %",
                "Abs Dev %",
                "Z-Score",
                "Assessment",
            ]);

        for stat in &report.first_digits {
            let abs_dev = (stat.actual_proportion - stat.expected_proportion).abs() * 100.0;
            let (status_text, status_color) = if stat.is_anomalous {
                ("ANOMALOUS SPIKE", Color::Red)
            } else {
                ("Conforming", Color::Green)
            };

            digit_table.add_row(vec![
                Cell::new(format!("{}", stat.digit)),
                Cell::new(format!("{}", stat.observed_count)),
                Cell::new(format!("{:.2}%", stat.actual_proportion * 100.0)),
                Cell::new(format!("{:.2}%", stat.expected_proportion * 100.0)),
                Cell::new(format!("{:.2}%", abs_dev)),
                Cell::new(format!("{:.2}", stat.z_score)),
                Cell::new(status_text).fg(status_color),
            ]);
        }
        println!("{digit_table}\n");
    }
}

pub fn print_findings_table(title: &str, findings: &[audit_core::models::AuditFinding]) {
    println!("\n================================================================================");
    println!("   {}", title.to_uppercase());
    println!("================================================================================\n");

    if findings.is_empty() {
        println!("No exceptions or control breaches identified.\n");
        return;
    }

    println!("Exceptions Identified: {}\n", findings.len());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Severity", "Rule", "Ref / Tx ID", "Line", "Exposure", "Details"]);

    for f in findings {
        let sev_color = match f.severity {
            RiskSeverity::Critical => Color::Red,
            RiskSeverity::High => Color::DarkRed,
            RiskSeverity::Medium => Color::Yellow,
            RiskSeverity::Low => Color::Blue,
        };

        let tx_str = f.transaction_id.as_deref().unwrap_or("-");
        let line_str = f.raw_line.map(|l| l.to_string()).unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            Cell::new(format!("{}", f.severity)).fg(sev_color),
            Cell::new(&f.rule),
            Cell::new(tx_str),
            Cell::new(line_str),
            Cell::new(format!("${:.2}", f.exposure_amount)),
            Cell::new(&f.description),
        ]);
    }

    println!("{table}\n");
}
