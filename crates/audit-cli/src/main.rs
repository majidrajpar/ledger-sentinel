mod display;
mod reader;

use std::path::PathBuf;
use std::process::ExitCode;
use clap::{Parser, Subcommand, ValueEnum};
use audit_core::benford::analyze_benford;
use audit_core::duplicates::detect_fuzzy_duplicates;
use audit_core::sod::detect_sod_breaches;
use audit_core::structuring::{detect_structuring, StructuringConfig};

#[derive(Parser)]
#[command(name = "ledger-sentinel")]
#[command(author = "Majid Mumtaz")]
#[command(version = "0.1.0")]
#[command(about = "Fast, local-first Continuous Controls Monitoring & forensic ledger audit engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all continuous control and forensic tests across the dataset
    Scan {
        /// Path to the CSV ledger or transaction export
        file: PathBuf,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,

        /// Structuring threshold limit (e.g. 5000 for delegation limit checks)
        #[arg(short, long, default_value_t = 5000.0)]
        threshold: f64,

        /// Maximum detailed anomaly rows to display in terminal
        #[arg(short, long, default_value_t = 25)]
        limit: usize,

        /// Return exit code 1 if any Critical severity violations are found
        #[arg(long, default_value_t = false)]
        fail_on_critical: bool,
    },

    /// Run Nigrini-grade Benford's Law tests on transaction amounts
    Benford {
        /// Path to the CSV ledger or transaction export
        file: PathBuf,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },

    /// Detect structured/split transactions clustered just below approval limits
    Structuring {
        /// Path to the CSV ledger or transaction export
        file: PathBuf,

        /// Structuring threshold limit
        #[arg(short, long, default_value_t = 5000.0)]
        threshold: f64,

        /// Clustering margin percentage below threshold (e.g. 0.10 for 90-100%)
        #[arg(short, long, default_value_t = 0.10)]
        margin: f64,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },

    /// Detect Segregation of Duties (maker-checker / creator-approver) collisions
    Sod {
        /// Path to the CSV ledger or transaction export
        file: PathBuf,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },

    /// Detect duplicate payments with fuzzy invoice reference normalization
    Duplicates {
        /// Path to the CSV ledger or transaction export
        file: PathBuf,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            file,
            format,
            threshold,
            limit,
            fail_on_critical,
        } => {
            let (transactions, _mapping) = match reader::read_transactions_from_csv(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Error reading CSV file '{}': {}", file.display(), e);
                    return ExitCode::FAILURE;
                }
            };

            let struct_cfg = StructuringConfig {
                threshold,
                margin_pct: 0.10,
                min_repetitions: 2,
            };

            let report = audit_core::run_full_audit(&transactions, Some(struct_cfg));

            match format {
                OutputFormat::Table => {
                    display::print_report_table(&report, limit);
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&report).unwrap());
                }
            }

            if fail_on_critical && report.summary_by_severity.critical_count > 0 {
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }

        Commands::Benford { file, format } => {
            let (transactions, _) = match reader::read_transactions_from_csv(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Error reading CSV file: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            let amounts: Vec<f64> = transactions.iter().map(|t| t.amount).collect();
            let (report, findings) = analyze_benford(&amounts);

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                        "benford_analysis": report,
                        "findings": findings
                    })).unwrap());
                }
                OutputFormat::Table => {
                    println!("\n=== BENFORD'S LAW EVALUATION ===");
                    println!("Eligibility:       {}", if report.is_eligible { "ELIGIBLE" } else { "INELIGIBLE" });
                    if let Some(reason) = &report.ineligibility_reason {
                        println!("Reason:            {}", reason);
                    }
                    println!("Population Size:   {}", report.population_size);
                    println!("Mean Abs Dev:      {:.4}", report.mad);
                    println!("Conformity:        {}", report.conformity);
                    println!("\n--- FIRST DIGIT DISTRIBUTION ---");
                    for stat in &report.first_digits {
                        let flag = if stat.is_anomalous { "[ANOMALOUS SPIKE]" } else { "                 " };
                        println!(
                            "Digit {}: Observed {:>6} ({:>5.2}%) | Expected {:>5.2}% | z-score: {:>5.2} {}",
                            stat.digit, stat.observed_count, stat.actual_proportion * 100.0,
                            stat.expected_proportion * 100.0, stat.z_score, flag
                        );
                    }
                }
            }
            ExitCode::SUCCESS
        }

        Commands::Structuring {
            file,
            threshold,
            margin,
            format,
        } => {
            let (transactions, _) = match reader::read_transactions_from_csv(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Error reading CSV file: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            let cfg = StructuringConfig {
                threshold,
                margin_pct: margin,
                min_repetitions: 2,
            };

            let findings = detect_structuring(&transactions, &cfg);

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&findings).unwrap());
                }
                OutputFormat::Table => {
                    println!("\n=== SPLIT PURCHASE / STRUCTURING VIOLATIONS ===");
                    println!("Threshold Limit:   ${:.2}", threshold);
                    println!("Bracket Window:    ${:.2} - ${:.2}", threshold * (1.0 - margin), threshold);
                    println!("Violations Found:  {}\n", findings.len());
                    for f in &findings {
                        println!("[{}] {} -> Exposure: ${:.2}", f.severity, f.title, f.exposure_amount);
                        println!("     Details: {}", f.description);
                    }
                }
            }
            ExitCode::SUCCESS
        }

        Commands::Sod { file, format } => {
            let (transactions, _) = match reader::read_transactions_from_csv(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Error reading CSV file: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            let findings = detect_sod_breaches(&transactions);

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&findings).unwrap());
                }
                OutputFormat::Table => {
                    println!("\n=== SEGREGATION OF DUTIES (MAKER-CHECKER) BREACHES ===");
                    println!("Violations Found:  {}\n", findings.len());
                    for f in &findings {
                        println!("[{}] {} | Amount: ${:.2}", f.severity, f.title, f.exposure_amount);
                        println!("     Details: {}", f.description);
                    }
                }
            }
            ExitCode::SUCCESS
        }

        Commands::Duplicates { file, format } => {
            let (transactions, _) = match reader::read_transactions_from_csv(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Error reading CSV file: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            let findings = detect_fuzzy_duplicates(&transactions);

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&findings).unwrap());
                }
                OutputFormat::Table => {
                    println!("\n=== POTENTIAL DUPLICATE DISBURSEMENTS ===");
                    println!("Violations Found:  {}\n", findings.len());
                    for f in &findings {
                        println!("[{}] {} | Amount: ${:.2}", f.severity, f.title, f.exposure_amount);
                        println!("     Details: {}", f.description);
                    }
                }
            }
            ExitCode::SUCCESS
        }
    }
}
