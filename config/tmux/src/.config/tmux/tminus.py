from datetime import datetime

target_date = datetime(2026, 1, 19, 9)
now = datetime.now()
delta = target_date - now

total_seconds = int(delta.total_seconds())

if total_seconds < 0:
    print("T-MINUS 0D 0H 0M 0S")
else:
    days = total_seconds // 86400
    hours = (total_seconds % 86400) // 3600
    minutes = (total_seconds % 3600) // 60
    seconds = total_seconds % 60
    
    print(f"T-MINUS {days}D {hours}H {minutes}M {seconds}S")
