use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskSeverity::Low => write!(f, "LOW"),
            RiskSeverity::Medium => write!(f, "MEDIUM"),
            RiskSeverity::High => write!(f, "HIGH"),
            RiskSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub amount: f64,
    pub date: Option<String>,
    pub vendor: Option<String>,
    pub account: Option<String>,
    pub created_by: Option<String>,
    pub approved_by: Option<String>,
    pub entry_type: Option<String>,
    pub description: Option<String>,
    pub raw_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub rule: String,
    pub severity: RiskSeverity,
    pub title: String,
    pub description: String,
    pub transaction_id: Option<String>,
    pub exposure_amount: f64,
    pub raw_line: Option<usize>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub total_rows_scanned: usize,
    pub total_exposure_at_risk: f64,
    pub findings: Vec<AuditFinding>,
    pub summary_by_rule: Vec<RuleSummary>,
    pub summary_by_severity: SeveritySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSummary {
    pub rule: String,
    pub count: usize,
    pub total_exposure: f64,
    pub highest_severity: RiskSeverity,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeveritySummary {
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
}
