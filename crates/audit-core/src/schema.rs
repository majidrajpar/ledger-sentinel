use crate::models::Transaction;

#[derive(Debug, Clone, Default)]
pub struct ColumnMapping {
    pub id_col: Option<usize>,
    pub amount_col: Option<usize>,
    pub date_col: Option<usize>,
    pub vendor_col: Option<usize>,
    pub account_col: Option<usize>,
    pub created_by_col: Option<usize>,
    pub approved_by_col: Option<usize>,
    pub entry_type_col: Option<usize>,
    pub description_col: Option<usize>,
}

impl ColumnMapping {
    pub fn detect_from_headers(headers: &[&str]) -> Self {
        let mut mapping = ColumnMapping::default();

        let id_aliases = ["id", "trans_id", "doc_num", "voucher", "voucher_no", "inv_num", "invoice_no", "ref", "reference", "doc_number", "trans_num"];
        let amount_aliases = ["amount", "amt", "total", "net_amount", "debit", "line_amount", "val", "value", "payment_amount", "invoice_total"];
        let date_aliases = ["date", "posting_date", "doc_date", "trans_date", "gl_date", "effective_date", "entry_date"];
        let vendor_aliases = ["vendor", "vendor_id", "vendor_name", "supplier", "payee", "creditor", "party", "beneficiary"];
        let account_aliases = ["account", "acct", "gl_account", "cost_center", "account_code", "account_num"];
        let created_by_aliases = ["created_by", "maker", "entered_by", "user", "user_id", "operator", "prepared_by"];
        let approved_by_aliases = ["approved_by", "checker", "posted_by", "authorizer", "signed_by", "authorized_by"];
        let entry_type_aliases = ["type", "entry_type", "source", "journal_type", "doc_type", "category", "journal_source"];
        let description_aliases = ["description", "desc", "narration", "line_item", "memo", "details"];

        for (idx, header) in headers.iter().enumerate() {
            let clean = header.trim().to_lowercase().replace([' ', '-', '.'], "_");

            if mapping.amount_col.is_none() && amount_aliases.iter().any(|&a| clean == a || clean.contains(a)) {
                mapping.amount_col = Some(idx);
            } else if mapping.id_col.is_none() && id_aliases.iter().any(|&a| clean == a || clean.contains(a)) {
                mapping.id_col = Some(idx);
            } else if mapping.date_col.is_none() && date_aliases.iter().any(|&a| clean == a || clean.contains(a)) {
                mapping.date_col = Some(idx);
            } else if mapping.vendor_col.is_none() && vendor_aliases.iter().any(|&a| clean == a || clean.contains(a)) {
                mapping.vendor_col = Some(idx);
            } else if mapping.created_by_col.is_none() && created_by_aliases.iter().any(|&a| clean == a || clean.contains(a)) {
                mapping.created_by_col = Some(idx);
            } else if mapping.approved_by_col.is_none() && approved_by_aliases.iter().any(|&a| clean == a || clean.contains(a)) {
                mapping.approved_by_col = Some(idx);
            } else if mapping.account_col.is_none() && account_aliases.iter().any(|&a| clean == a || clean.contains(a)) {
                mapping.account_col = Some(idx);
            } else if mapping.entry_type_col.is_none() && entry_type_aliases.iter().any(|&a| clean == a || clean.contains(a)) {
                mapping.entry_type_col = Some(idx);
            } else if mapping.description_col.is_none() && description_aliases.iter().any(|&a| clean == a || clean.contains(a)) {
                mapping.description_col = Some(idx);
            }
        }

        mapping
    }

    pub fn parse_row(&self, row: &[&str], line_number: usize) -> Option<Transaction> {
        let amount_str = self.amount_col.and_then(|idx| row.get(idx))?;
        let clean_amount = amount_str
            .trim()
            .replace(['$', '€', '£', ',', ' '], "");
        
        let amount = clean_amount.parse::<f64>().ok()?;

        let id = self.id_col
            .and_then(|idx| row.get(idx))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| format!("ROW-{}", line_number));

        let date = self.date_col
            .and_then(|idx| row.get(idx))
            .map(|s| s.trim().to_string());

        let vendor = self.vendor_col
            .and_then(|idx| row.get(idx))
            .map(|s| s.trim().to_string());

        let account = self.account_col
            .and_then(|idx| row.get(idx))
            .map(|s| s.trim().to_string());

        let created_by = self.created_by_col
            .and_then(|idx| row.get(idx))
            .map(|s| s.trim().to_string());

        let approved_by = self.approved_by_col
            .and_then(|idx| row.get(idx))
            .map(|s| s.trim().to_string());

        let entry_type = self.entry_type_col
            .and_then(|idx| row.get(idx))
            .map(|s| s.trim().to_string());

        let description = self.description_col
            .and_then(|idx| row.get(idx))
            .map(|s| s.trim().to_string());

        Some(Transaction {
            id,
            amount,
            date,
            vendor,
            account,
            created_by,
            approved_by,
            entry_type,
            description,
            raw_line: line_number,
        })
    }
}
