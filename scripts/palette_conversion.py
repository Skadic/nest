import re

pattern = re.compile(r'palScreen(\[\S+\]) = olc::Pixel\(([0-9]{1,3}), ([0-9]{1,3}), ([0-9]{1,3})\);')

inFile = open("palette_data.txt")
lines = inFile.readlines()
inFile.close()

for line in lines:
    matches = pattern.findall(line)
    with open("palette_out.txt", "a") as outFile:
        for match in matches:
            outFile.write('self.palette_screen%s = Rgba([%s, %s, %s, 255]);' % (match[0], match[1], match[2], match[3]))
        outFile.write('\n')
    