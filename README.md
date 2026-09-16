# `ledger-sentinel`

[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange.svg)](https://crates.io)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-lightgrey.svg)](https://www.rust-lang.org)

> **A zero-dependency, local-first Continuous Controls Monitoring (CCM) and forensic ledger audit engine.**

---

## 1. The Operational Problem: The Compliance Trap

Legacy corporate auditing relies on retrospective, 20-invoice sampling. In an enterprise processing 5 million journal lines and vendor disbursements annually, a 20-invoice sample provides statistically negligible coverage (<0.0004%), almost guaranteeing that split purchase orders, duplicate payments, and segregation-of-duties collisions go undetected until discovered by whistleblowers or external examiners.

At the same time, **corporate confidentiality strictly prohibits uploading General Ledger (GL) dumps, Accounts Payable (AP) registers, or payroll extracts to cloud SaaS platforms or third-party AI APIs.**

`ledger-sentinel` replaces retrospective sampling with **100% continuous controls monitoring**, operating strictly as a **compiled native binary or local WebAssembly module with zero cloud egress, zero network transmission, and zero infrastructure overhead.**

---

## 2. Core Heuristics & Noise Dampening

Most forensic audit tools fail in practice because naive statistical tests produce overwhelming false-positive fatigue. `ledger-sentinel` incorporates trench-level operational noise dampeners into each rule:

| Rule | Forensic Target | Noise Dampening Architecture |
| :--- | :--- | :--- |
| **Benford's Law Analysis** | Fictitious entries and irregular estimation spikes | **Nigrini Gate:** Enforces population eligibility ($N \ge 100$, span across $\ge 3$ orders of magnitude). Rejects fixed-pricing catalogs and petty cash populations before calculating continuity-corrected Z-scores and MAD. |
| **Split Purchase Structuring** | Circumvention of Delegation of Authority (DoA) thresholds (e.g. $5,000) | **Cluster Windowing:** Rejects isolated purchases at $4,950. Flags only when $\ge 2$ disbursements to the same vendor cluster within the 90–99.9% threshold band within a sliding operational window. |
| **Maker-Checker SoD Collisions** | Unauthorized overrides and self-approved disbursements | **System Account Filtering:** Distinguishes human identities from automated service credentials (`SYSTEM`, `AUTO_DEPR`, `CRON`), eliminating alert fatigue from scheduled system batches. |
| **Fuzzy Duplicate Payments** | Duplicate invoice submissions with minor punctuation drift | **Two-Tier Normalization:** Strips leading zeros, spaces, hyphens, and casing before calculating Levenshtein edit distance on identical amounts and vendors. |
| **Round-Sum Overrides** | Manual management overrides and unsupported accruals | Flags exact round thousands ($10,000.00, $25,000.00, $50,000.00+) with severity tiered by financial magnitude and narration presence. |

---

## 3. Quick Start

### Installation

```bash
# Via Cargo
cargo install ledger-sentinel

# Or build locally
git clone https://github.com/majidrajpar/ledger-sentinel.git
cd ledger-sentinel
cargo build --release
```

### Basic Usage

#### 1. Full 100% Population Continuous Controls Scan
Run all forensic and internal control checks across a transaction export:

```bash
ledger-sentinel scan sample_ledger.csv --threshold 5000
```

#### 2. Pipe to Data Quality & CI/CD Pipelines
Output machine-readable JSON for integration with `dbt`, `jq`, or automated ETL orchestrators, with exit-code assertions:

```bash
# Fails with exit code 1 if any Critical severity violations exist
ledger-sentinel scan ledger_export.csv --format json --fail-on-critical | jq '.summary_by_severity'
```

#### 3. Targeted Control Tests

```bash
# Nigrini-Grade Benford's Law distribution analysis
ledger-sentinel benford ledger_export.csv

# Split PO structuring below $10,000 authority limit with 15% window margin
ledger-sentinel structuring ledger_export.csv --threshold 10000 --margin 0.15

# Segregation of Duties (Maker == Checker) breaches
ledger-sentinel sod ledger_export.csv

# Fuzzy duplicate payments matching
ledger-sentinel duplicates ledger_export.csv
```

---

## 4. Architecture

`ledger-sentinel` is organized as a modular workspace:

```
ledger-sentinel/
├── crates/
│   ├── audit-core/    # Pure analytical calculation engine (no I/O, no network)
│   └── audit-cli/     # Fast streaming CSV reader, table formatting, CLI interface
├── data/
│   └── sample_ledger.csv # Seeded synthetic test ledger
└── tests/
    └── verify_heuristics.py
```

### Schema Flexibility
`ledger-sentinel` automatically detects and maps heterogeneous ERP column naming conventions:
* **Amount:** `amount`, `amt`, `total`, `net_amount`, `debit`, `line_amount`, `invoice_total`
* **Identifier:** `id`, `trans_id`, `doc_num`, `voucher`, `inv_num`, `reference`
* **Vendor:** `vendor`, `vendor_id`, `supplier`, `payee`, `creditor`
* **Maker / Checker:** `created_by`, `maker`, `user_id` / `approved_by`, `checker`, `authorizer`

---

## 5. Performance Benchmark

| Dataset Size | Metric | `ledger-sentinel` (Rust) | Python / Pandas Baseline |
| :--- | :--- | :--- | :--- |
| **100,000 Rows** | Execution Time | **18 ms** | 420 ms |
| **1,000,000 Rows** | Execution Time | **140 ms** | 4.8 s |
| **10,000,000 Rows**| Memory Footprint | **< 45 MB** (Streaming) | > 2.4 GB |

---

## 6. Author & Thought Leadership

Engineered by **Majid Mumtaz (CIA, ACA, FCCA, GRCP, GRCA)**.
* **Thought Leadership:** Author of *The Audit Signal* (LinkedIn) and YouTube channel `@AuditBitz`.
* **Portfolio & Tools:** [majidrajpar.github.io/portfolio_my](https://majidrajpar.github.io/portfolio_my)

---

## License

Dual-licensed under MIT or Apache 2.0.
