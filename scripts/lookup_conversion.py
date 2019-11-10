import re

pattern = re.compile(r'{\s*"(\S+?)",\s*&a::(\S+?),\s*&a::(\S+?),\s*(\S+)\s*}')

inFile = open("scripts/cpp_lookup_table.txt")
lines = inFile.readlines()
inFile.close()

for line in lines:
    matches = pattern.findall(line)
    with open("scripts/out.txt", "a") as outFile:
        outFile.write('vec![')
        for match in matches:
            outFile.write('Instruction::new("%s", Self::%s, Self::%s, %s), ' % (match[0], match[1], match[2], match[3]))
        outFile.write('],\n')
    