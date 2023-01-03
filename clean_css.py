with open("assets/katex.css") as file, open("katex.css", 'w') as out:
    for line in file:
        if "woff2" in line:
            out.write(line.split(',')[0] + ';')
        else:
            out.write(line)