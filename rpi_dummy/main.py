import serial
import time

s = serial.Serial('/dev/pts/5')

lines = """8d4785bf990cad1870688b920bb0
5d4785bff99ec3
a0000f1ceb099b1e618c32eaa723
8d4785bf990cad1870648bda51b0
8d4785bf990cad18706c8c55ed94
8d4785bf5879e1a90a196003be04
a0000f1eff5e27207ff49463148a
a0000f1eaee0000000000046c19f
28000a3e0aff58"""

for line in lines.split('\n'):
    for b in bytes(line + chr(10), 'utf-8'):
        print(b, ' ', end="")
    print()
    time.sleep(0.1)
    s.write(bytes(line + chr(10), 'utf-8'))

s.close()
