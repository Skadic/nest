import re

pattern = re.compile(r'{\s*"(\S+?)",\s*&a::(\S+?),\s*&a::(\S+?),\s*(\S+)\s*}')

inFile = open("scripts/cpp_lookup_table.txt")
lines = inFile.readlines()
inFile.close()

for line in lines:
    matches = pattern.findall(line)
    with open("scripts/out.txt", "a") as outFile:
        for match in matches:
            outFile.write('Instruction::new("%s", Olc6502::%s, Olc6502::%s, %s), ' % (match[0], match[1], match[2], match[3]))
        outFile.write('\n')
    