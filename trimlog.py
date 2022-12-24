import sys

f = open(sys.argv[1])

for line in f.readlines():
    print(f"{line[:5]}{line[16:19]}{line[47:]}", end="")
