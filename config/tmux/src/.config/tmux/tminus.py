from datetime import datetime

now = datetime.now()

# Calculate next occurrence of February 27th
birthday_year = now.year
target_date = datetime(birthday_year, 2, 27)

# If birthday has passed this year, target next year
if now > target_date:
    target_date = datetime(birthday_year + 1, 2, 27)

# Check if today is the birthday
if now.date() == target_date.date():
    print("T-ZERO")
else:
    delta = target_date - now
    total_seconds = int(delta.total_seconds())
    
    if total_seconds < 0:
        print("T-ZERO")
    else:
        days = total_seconds // 86400
        hours = (total_seconds % 86400) // 3600
        minutes = (total_seconds % 3600) // 60
        seconds = total_seconds % 60
        
        print(f"T-MINUS {days}D {hours}H {minutes}M {seconds}S")
