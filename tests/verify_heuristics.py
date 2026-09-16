import csv
import math
from collections import defaultdict

csv_path = r"C:\Users\sorat\Desktop\Coding\portfolio_my\src\rust\auditctl\data\sample_ledger.csv"

with open(csv_path, "r", encoding="utf-8") as f:
    reader = csv.DictReader(f)
    rows = list(reader)

print(f"Total Rows Scanned: {len(rows)}")

# 1. Benford 1st digit
amounts = [float(r["Amount"]) for r in rows if float(r["Amount"]) >= 1.0]
digits = [int(str(a).lstrip("0.")[0]) for a in amounts if str(a).lstrip("0.")[0] in "123456789"]
first_counts = defaultdict(int)
for d in digits:
    first_counts[d] += 1

total_n = len(digits)
mad = 0.0
print("\n--- BENFORD'S LAW (FIRST DIGIT) ---")
for d in range(1, 10):
    observed = first_counts[d]
    actual_pct = (observed / total_n) * 100.0
    expected_pct = math.log10(1 + 1/d) * 100.0
    abs_dev = abs(actual_pct - expected_pct)
    mad += abs_dev / 100.0
    print(f"Digit {d}: Observed {observed:>4} ({actual_pct:>5.1f}%) | Expected {expected_pct:>5.1f}% | Dev {abs_dev:>4.1f}%")

mad /= 9.0
print(f"Mean Absolute Deviation (MAD): {mad:.4f} -> Conformity: {'Close conformity' if mad <= 0.006 else 'Acceptable' if mad <= 0.012 else 'Marginally acceptable' if mad <= 0.015 else 'Nonconforming'}")

# 2. Structuring below $5,000 threshold
print("\n--- STRUCTURING / SPLIT INVOICES (Threshold $5,000 | Bracket $4,500 - $5,000) ---")
candidates = defaultdict(list)
for r in rows:
    amt = float(r["Amount"])
    if 4500.0 <= amt < 5000.0:
        candidates[r["Vendor_Name"]].append(r)

for vendor, items in candidates.items():
    if len(items) >= 2:
        tot = sum(float(x["Amount"]) for x in items)
        print(f"[CRITICAL] Vendor '{vendor}': {len(items)} transactions totaling ${tot:.2f}")
        for it in items:
            print(f"   - Ref {it['Invoice_Number']}: ${float(it['Amount']):.2f} on {it['Posting_Date']} by {it['Created_By']}")

# 3. Segregation of Duties (Maker == Checker)
print("\n--- SEGREGATION OF DUTIES BREACHES (Maker == Checker) ---")
for r in rows:
    maker = r["Created_By"].strip()
    checker = r["Approved_By"].strip()
    if maker and checker and maker.lower() == checker.lower():
        if not any(sys in maker.upper() for sys in ["SYSTEM", "AUTO", "CRON"]):
            amt = float(r["Amount"])
            print(f"[CRITICAL] Ref {r['Invoice_Number']}: User '{maker}' self-approved entry of ${amt:.2f} for account '{r['GL_Account']}'")

# 4. Fuzzy Duplicate Payments
print("\n--- FUZZY DUPLICATE DISBURSEMENTS ---")
def norm_ref(s):
    return "".join(c for c in s if c.isalnum()).upper().lstrip("0")

grouped = defaultdict(list)
for r in rows:
    grouped[(r["Vendor_Name"], round(float(r["Amount"]), 2))].append(r)

for (v, amt), items in grouped.items():
    if len(items) >= 2:
        for i in range(len(items)):
            for j in range(i+1, len(items)):
                n1 = norm_ref(items[i]["Invoice_Number"])
                n2 = norm_ref(items[j]["Invoice_Number"])
                if n1 == n2:
                    print(f"[HIGH] Vendor '{v}' | Amount ${amt:.2f}: Ref '{items[i]['Invoice_Number']}' vs Ref '{items[j]['Invoice_Number']}'")

# 5. Round Sum Overrides >= $25,000
print("\n--- ROUND-SUM ENTRIES >= $25,000 ---")
for r in rows:
    amt = float(r["Amount"])
    if amt >= 25000.0 and amt % 1000 == 0:
        print(f"[MEDIUM] Ref {r['Invoice_Number']}: Exact round amount ${amt:.2f} to '{r['Vendor_Name']}' | Narration: {r['Description']}")
