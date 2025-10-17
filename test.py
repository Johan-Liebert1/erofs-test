l=[117, 115, 114, 47, 98, 105, 110]

print("".join(chr(i) if i < 127 and i != 0 else "|" for i in l))
