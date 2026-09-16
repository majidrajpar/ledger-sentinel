use std::fs::File;
use std::path::Path;
use audit_core::models::Transaction;
use audit_core::schema::ColumnMapping;

pub fn read_transactions_from_csv<P: AsRef<Path>>(
    path: P,
) -> Result<(Vec<Transaction>, ColumnMapping), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    let headers_record = rdr.headers()?.clone();
    let header_strs: Vec<&str> = headers_record.iter().collect();
    let mapping = ColumnMapping::detect_from_headers(&header_strs);

    let mut transactions = Vec::new();
    let mut raw_record = csv::StringRecord::new();
    let mut line_no = 2; // header is line 1

    while rdr.read_record(&mut raw_record)? {
        let fields: Vec<&str> = raw_record.iter().collect();
        if let Some(tx) = mapping.parse_row(&fields, line_no) {
            transactions.push(tx);
        }
        line_no += 1;
    }

    Ok((transactions, mapping))
}
