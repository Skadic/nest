import re

pattern = re.compile(r'{\s*"(\S+?)",\s*&a::(\S+?),\s*&a::(\S+?),\s*(\S+)\s*}')


with open("scripts/cpp_lookup_table.txt") as inFile:
    lines = inFile.readlines()

    for line in lines:
        matches = pattern.findall()
        with open("scripts/out.txt", "w") as outFile:
            for match in matches:
                print("Instruction {}")
    