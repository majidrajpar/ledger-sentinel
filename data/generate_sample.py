import random
import csv
import os

os.makedirs(r"C:\Users\sorat\Desktop\Coding\portfolio_my\src\rust\auditctl\data", exist_ok=True)
csv_path = r"C:\Users\sorat\Desktop\Coding\portfolio_my\src\rust\auditctl\data\sample_ledger.csv"

random.seed(42)

# Vendors and users
vendors = ["TechSupplies Inc", "CloudScale Networks", "OfficeDepot Co", "Delta Freight", "Metro Facilities", "Apex Consulting LLC", "Global Logistics Ltd"]
accounts = ["6010-Office Expense", "6020-IT Cloud Infra", "6030-Consulting Fees", "6040-Freight & Delivery", "6050-Maintenance"]
makers = ["alice.smith", "bob.jones", "charlie.brown", "john.doe", "emily.watson", "SYSTEM_CRON", "AUTO_INTERFACE"]
checkers = ["diana.prince", "frank.castle", "bruce.wayne", "john.doe", "clark.kent", "SYSTEM_CRON"]

rows = []

# 1. Generate 2,000 baseline natural transactions conforming roughly to log-normal / Benford
for i in range(1, 2001):
    doc_id = f"INV-{10000 + i}"
    # Log-normal distribution naturally conforms to Benford's Law
    amt = round(random.expovariate(1.0 / 450.0) + random.uniform(10.0, 5000.0), 2)
    amt = max(1.0, amt)
    
    vendor = random.choice(vendors[:5])
    account = random.choice(accounts)
    maker = random.choice(["alice.smith", "bob.jones", "charlie.brown", "emily.watson"])
    checker = random.choice(["diana.prince", "frank.castle", "bruce.wayne"])
    date = f"2026-03-{random.randint(1, 28):02d}"
    desc = f"Payment for standard services {doc_id}"

    rows.append([doc_id, amt, date, vendor, account, maker, checker, "AP_INVOICE", desc])

# 2. Seed Structured Split Purchases (Apex Consulting LLC under $5,000 threshold)
for k in range(3):
    split_id = f"APX-SPLIT-0{k+1}"
    split_amt = round(4900.0 + random.uniform(10.0, 85.0), 2)
    rows.append([split_id, split_amt, "2026-03-12", "Apex Consulting LLC", "6030-Consulting Fees", "charlie.brown", "frank.castle", "AP_INVOICE", "Advisory Phase installment"])

# 3. Seed Critical SoD Collisions (maker == checker == john.doe)
rows.append(["MAN-JV-9901", 15400.00, "2026-03-15", "Apex Consulting LLC", "6030-Consulting Fees", "john.doe", "john.doe", "MANUAL_JV", "Special management advisory accrual"])
rows.append(["MAN-JV-9902", 3200.00, "2026-03-18", "TechSupplies Inc", "6010-Office Expense", "john.doe", "john.doe", "MANUAL_JV", "Expedited hardware adjustment"])

# 4. Seed Duplicate Payment (Global Logistics Ltd: INV-88204 vs INV88204)
rows.append(["INV-88204", 8750.50, "2026-03-05", "Global Logistics Ltd", "6040-Freight & Delivery", "alice.smith", "diana.prince", "AP_INVOICE", "International courier freight charges"])
rows.append(["INV88204", 8750.50, "2026-03-19", "Global Logistics Ltd", "6040-Freight & Delivery", "bob.jones", "frank.castle", "AP_INVOICE", "Courier freight invoice re-submission"])

# 5. Seed Round-Sum Manual Overrides
rows.append(["MAN-JV-8800", 50000.00, "2026-03-20", "Delta Freight", "6040-Freight & Delivery", "emily.watson", "bruce.wayne", "MANUAL_JV", "Q1 Year-end bulk logistics true-up"])
rows.append(["MAN-JV-8801", 25000.00, "2026-03-22", "CloudScale Networks", "6020-IT Cloud Infra", "bob.jones", "diana.prince", "MANUAL_JV", "Annual server capacity reservation"])

with open(csv_path, "w", newline="", encoding="utf-8") as f:
    writer = csv.writer(f)
    writer.writerow(["Invoice_Number", "Amount", "Posting_Date", "Vendor_Name", "GL_Account", "Created_By", "Approved_By", "Entry_Type", "Description"])
    writer.writerows(rows)

print(f"Generated {len(rows)} rows into {csv_path}")
