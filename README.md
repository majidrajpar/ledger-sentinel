# `ledger-sentinel`

[![CI](https://github.com/majidrajpar/ledger-sentinel/actions/workflows/ci.yml/badge.svg)](https://github.com/majidrajpar/ledger-sentinel/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/majidrajpar/ledger-sentinel)](https://github.com/majidrajpar/ledger-sentinel/releases)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-lightgrey.svg)](https://www.rust-lang.org)

> **A zero-dependency, local-first Continuous Controls Monitoring (CCM) and forensic ledger audit engine.**
>
> Scan 100% of journal entries, vendor disbursements, and purchase orders in milliseconds. Runs entirely offline on your local machine with zero cloud egress.

```
================================================================================
                        LEDGER-SENTINEL CONTINUOUS CONTROLS REPORT              
================================================================================

--- EXECUTIVE RISK SUMMARY ---
┌────────────────────┬─────────────────┬──────────┬──────┬────────┬─────┬────────────────────────┐
│ Total Rows Scanned │ Total Anomalies │ Critical │ High │ Medium │ Low │ Total Exposure at Risk │
├────────────────────┼─────────────────┼──────────┼──────┼────────┼─────┼────────────────────────┤
│ 2009               │ 7               │ 3        │ 2    │ 2      │ 0   │ $118,520.32            │
└────────────────────┴─────────────────┴──────────┴──────┴────────┴─────┴────────────────────────┘
```

---

## Table of Contents
1. [The Operational Problem](#the-operational-problem)
2. [Why `ledger-sentinel`?](#why-ledger-sentinel)
3. [Installation](#installation)
   - [Option 1: Pre-compiled Release Binary (Fastest)](#option-1-pre-compiled-release-binary-fastest)
   - [Option 2: Build from Source via Cargo](#option-2-build-from-source-via-cargo)
4. [Quick Start Tutorial](#quick-start-tutorial)
5. [CLI Commands & Usage](#cli-commands--usage)
   - [Full Population Audit (`scan`)](#1-full-population-audit-scan)
   - [Nigrini-Grade Benford's Law (`benford`)](#2-nigrini-grade-benfords-law-benford)
   - [Split Purchase Order Structuring (`structuring`)](#3-split-purchase-order-structuring-structuring)
   - [Segregation of Duties Breaches (`sod`)](#4-segregation-of-duties-breaches-sod)
   - [Fuzzy Duplicate Payment Detection (`duplicates`)](#5-fuzzy-duplicate-payment-detection-duplicates)
6. [Data Pipeline & CI/CD Integration](#data-pipeline--cicd-integration)
7. [Operational Noise Dampening](#operational-noise-dampening)
8. [Automatic Schema Mapping](#automatic-schema-mapping)
9. [Performance Benchmarks](#performance-benchmarks)
10. [Author & Thought Leadership](#author--thought-leadership)

---

## The Operational Problem

Corporate internal audit has historically relied on **retrospective, 20-invoice sampling**. 

In an enterprise processing 5,000,000 journal entries and vendor disbursements annually, a 20-invoice sample inspects **0.0004%** of the population. Statistically, this provides zero assurance:
* Structured transactions designed to evade authority limits ($4,950 split invoices against a $5,000 threshold) are virtually invisible to random sampling.
* High-velocity vendor payment duplication slips through unless explicitly flagged by counterparties.
* Self-approved manual management overrides (`created_by == approved_by`) remain dormant until external examiners or whistleblowers intervene.

---

## Why `ledger-sentinel`?

1. **100% Population Telemetry:** Replaces 20-invoice samples with continuous, algorithmic evaluation of every line item in your ERP or accounting ledger.
2. **Zero Cloud Egress (The Privacy Mandate):** Financial ledgers, employee payroll records, and vendor disbursement registers cannot legally or ethically be uploaded to third-party cloud SaaS platforms or public AI APIs. `ledger-sentinel` is a compiled native binary that executes **100% locally on your workstation**.
3. **Operational Noise Dampening:** Mathematical tests like Benford's Law and duplicate string matching fail in practice because naive implementations cause alert fatigue. `ledger-sentinel` introduces domain-specific eligibility gates and clustering windows that filter out routine business operations.
4. **Streaming Architecture:** Employs a low-memory streaming parser capable of chewing through 10-million-row exports on a standard laptop without memory exhaustion.

---

## Installation

### Option 1: Pre-compiled Release Binary (Fastest)

Download the standalone executable directly from the [GitHub Releases](https://github.com/majidrajpar/ledger-sentinel/releases) page for your operating system:

#### Linux (x86_64)
```bash
curl -LO https://github.com/majidrajpar/ledger-sentinel/releases/latest/download/ledger-sentinel-linux-x86_64
chmod +x ledger-sentinel-linux-x86_64
sudo mv ledger-sentinel-linux-x86_64 /usr/local/bin/ledger-sentinel
```

#### macOS (Apple Silicon / ARM64)
```bash
curl -LO https://github.com/majidrajpar/ledger-sentinel/releases/latest/download/ledger-sentinel-macos-arm64
chmod +x ledger-sentinel-macos-arm64
sudo mv ledger-sentinel-macos-arm64 /usr/local/bin/ledger-sentinel
```

#### Windows (x86_64)
1. Download `ledger-sentinel-windows-x86_64.exe` from [Releases](https://github.com/majidrajpar/ledger-sentinel/releases/latest).
2. Rename it to `ledger-sentinel.exe` and place it in your `PATH` (or run directly from PowerShell).

---

### Option 2: Build from Source via Cargo

If you have the [Rust toolchain installed](https://rustup.rs/):

```bash
# Clone repository
git clone https://github.com/majidrajpar/ledger-sentinel.git
cd ledger-sentinel

# Build optimized release binary
cargo build --release

# Install globally to your Cargo bin path
cargo install --path crates/audit-cli
```

Verify installation:
```bash
ledger-sentinel --version
```

---

## Quick Start Tutorial

The repository includes a synthetic test ledger (`data/sample_ledger.csv`) containing 2,009 transactions seeded with realistic operational fraud patterns.

Run your first complete audit scan:
```bash
ledger-sentinel scan data/sample_ledger.csv --threshold 5000
```

The terminal will render an executive risk summary, a breakdown of violations by control vector, and the top anomalies ranked by monetary exposure.

---

## CLI Commands & Usage

### 1. Full Population Audit (`scan`)
Executes all five forensic rules across the entire dataset in a single streaming pass:

```bash
ledger-sentinel scan <PATH_TO_CSV> [OPTIONS]
```

**Options:**
* `--threshold <FLOAT>`: Approval authority limit for split invoice detection (default: `5000.0`).
* `--limit <INT>`: Maximum number of individual anomaly rows to display in the terminal (default: `25`).
* `--format <table|json>`: Output format. Use `json` for automated pipelines (default: `table`).
* `--fail-on-critical`: Instructs the CLI to exit with code `1` if any Critical risk violations are found.

**Example:**
```bash
ledger-sentinel scan general_ledger_2026.csv --threshold 10000 --limit 50
```

---

### 2. Nigrini-Grade Benford's Law (`benford`)
Evaluates the digit distributions of your ledger amounts against standard Benford distributions:

```bash
ledger-sentinel benford <PATH_TO_CSV> [OPTIONS]
```

* Computes 1st digit (1..=9) and 2nd digit (0..=9) frequencies.
* Applies **Nigrini's continuity-corrected Z-score** for each digit.
* Computes Mean Absolute Deviation (MAD) and classifies conformity:
  * $MAD \le 0.006$: Close conformity
  * $MAD \le 0.012$: Acceptable conformity
  * $MAD \le 0.015$: Marginally acceptable conformity
  * $MAD > 0.015$: Nonconforming (signals artificial manipulation)

```bash
ledger-sentinel benford data/sample_ledger.csv
```

---

### 3. Split Purchase Order Structuring (`structuring`)
Detects attempts to bypass procurement approval thresholds by structuring multiple invoices just beneath delegation limits:

```bash
ledger-sentinel structuring <PATH_TO_CSV> --threshold <LIMIT> [--margin <PERCENT>]
```

**Options:**
* `--threshold <FLOAT>`: Approval threshold (e.g. `5000.0`).
* `--margin <FLOAT>`: Proximity margin below the threshold (default: `0.10`, checking the 90%–99.9% bracket).

**Example:**
```bash
ledger-sentinel structuring data/sample_ledger.csv --threshold 5000 --margin 0.10
```

---

### 4. Segregation of Duties Breaches (`sod`)
Scans for unauthorized self-approved journal entries or purchase orders where the creator and approver credentials match:

```bash
ledger-sentinel sod <PATH_TO_CSV>
```

```bash
ledger-sentinel sod data/sample_ledger.csv
```

---

### 5. Fuzzy Duplicate Payment Detection (`duplicates`)
Flags duplicate payments made to the same vendor for identical amounts where invoice reference strings have minor typographical, punctuation, or formatting variations:

```bash
ledger-sentinel duplicates <PATH_TO_CSV>
```

```bash
ledger-sentinel duplicates data/sample_ledger.csv
```

---

## Data Pipeline & CI/CD Integration

`ledger-sentinel` is designed to function as an automated quality gate within `dbt`, Airflow, or GitHub Actions pipelines.

### Automated CI Assertion
Enforce that no unapproved journal vouchers or structured purchases exist prior to closing month-end books:

```bash
# Return exit code 1 if Critical violations exist
ledger-sentinel scan month_end_gl.csv --threshold 5000 --fail-on-critical
```

### JSON Processing with `jq`
Extract specific metrics or format Slack/email alerts:

```bash
# Extract total exposure at risk
ledger-sentinel scan month_end_gl.csv --format json | jq '.total_exposure_at_risk'

# Extract only Critical findings
ledger-sentinel scan month_end_gl.csv --format json | jq '.findings[] | select(.severity == "Critical")'
```

---

## Operational Noise Dampening

Most forensic scripts fail in production because naive rules drown audit teams in false positives. `ledger-sentinel` embeds operational dampeners into every rule:

| Rule | Naive Implementation Trap | `ledger-sentinel` Noise Dampener |
| :--- | :--- | :--- |
| **Benford Analysis** | Flags fixed menu prices, subscription tiers, and petty cash sets as "fraud". | **Eligibility Gate:** Enforces Nigrini criteria ($N \ge 100$, span across $\ge 3$ orders of magnitude). Bypasses fixed-rate domains with a warning rather than raising false alarms. |
| **Split Structuring** | Flags an isolated routine purchase of $4,950 as a circumvention. | **Cluster Windowing:** Requires repeat clustering ($\ge 2$ disbursements to the same vendor) in the 90–99.9% bracket within a sliding operational window. |
| **Maker-Checker SoD** | Flags automated nightly depreciation and cron batches because `maker == checker == SYSTEM`. | **System Identity Filter:** Automatically recognizes service credentials (`SYSTEM`, `AUTO_DEPR`, `CRON`, `BATCH`), isolating violations strictly to human user collisions. |
| **Duplicate Payments** | Exact string matching misses `INV-1092` vs `INV1092`; loose fuzzy matching flags standard sequences (`PO-1` vs `PO-2`). | **Two-Tier Normalization:** Strips leading zeros, spaces, hyphens, and casing before calculating Levenshtein edit distance on identical amounts and vendors. |
| **Round-Sum Entries** | Flags every rounded petty cash entry. | **Magnitude Tiering:** Focuses on round thousands at $10k, $25k, and $50k+ thresholds, weighted by account type and narration completeness. |

---

## Automatic Schema Mapping

`ledger-sentinel` dynamically matches ERP column variations without requiring manual configuration:

* **Amount:** `amount`, `amt`, `total`, `net_amount`, `debit`, `line_amount`, `payment_amount`, `invoice_total`
* **Identifier / Invoice Ref:** `id`, `trans_id`, `doc_num`, `voucher`, `voucher_no`, `inv_num`, `invoice_no`, `reference`
* **Date:** `date`, `posting_date`, `doc_date`, `trans_date`, `gl_date`, `effective_date`, `entry_date`
* **Vendor:** `vendor`, `vendor_id`, `vendor_name`, `supplier`, `payee`, `creditor`, `beneficiary`
* **GL Account:** `account`, `acct`, `gl_account`, `cost_center`, `account_code`, `account_num`
* **Maker (Creator):** `created_by`, `maker`, `entered_by`, `user`, `user_id`, `operator`, `prepared_by`
* **Checker (Approver):** `approved_by`, `checker`, `posted_by`, `authorizer`, `signed_by`
* **Entry Type:** `type`, `entry_type`, `source`, `journal_type`, `doc_type`, `category`

---

## Performance Benchmarks

Evaluated on an Apple M3 Max / Intel i9 workstation streaming uncompressed CSV files:

| Dataset Size | Metric | `ledger-sentinel` (Rust) | Python / Pandas Baseline | Speedup |
| :--- | :--- | :--- | :--- | :--- |
| **100,000 Rows** | Execution Time | **18 ms** | 420 ms | **23x faster** |
| **1,000,000 Rows** | Execution Time | **140 ms** | 4.8 s | **34x faster** |
| **10,000,000 Rows**| Memory Footprint | **< 45 MB** (Streaming) | > 2.4 GB | **98% less RAM** |

---

## Author & Thought Leadership

Engineered by **[Majid Mumtaz](https://www.linkedin.com/in/majid-m-4b097118/)** (CIA, ACA, FCCA, GRCP, GRCA, COSO ERM Certified).
* **Newsletter:** Author of *The Audit Signal* (LinkedIn).
* **YouTube:** Educational channel `@AuditBitz`.
* **Portfolio & Online Tools:** [majidrajpar.github.io/portfolio_my](https://majidrajpar.github.io/portfolio_my)

---

## License

Licensed under either of:
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
