import hashlib

# This script helps create the default schema manually

while True:
    password = input("Password: ")
    if not password:
        break
    hashed = hashlib.sha256(password.encode()).hexdigest()
    print(f"Hash: {hashed}\n")