import serial
import time

s = serial.Serial('/dev/pts/3')

lines = """8D40621D58C386435CC412692AD6
8D40621D58C382D690C8AC2863A7
8D40621B58C386435CC412692AD6
8D40621B58C382D690C4AC2863A7
8D40621D994457958838765B284F
8D40621D9B06B6AF189400CBC33F
2A00516D492B80
8D40621D202CC371C32CE0576098
420100f47d2042e3a5a2412fdd9941e7fbc041e3388e3fecc039230aa98f408fc2f53d0000000077becf3f0ad7a3400ad7a3400000b040b81ea5400000b040b81ea5400102c832"""

for line in lines.split('\n'):
    for b in bytes(line + chr(10), 'utf-8'):
        print(b, ' ', end="")
    print()
    time.sleep(0.1)
    buf = bytearray.fromhex(line)
    s.write(buf + (b'\x00' * (128 - len(buf))))

s.close()
