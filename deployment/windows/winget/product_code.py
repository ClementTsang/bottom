import sys
from string import Template

args = sys.argv
template_file_path = args[1]
product_code = args[2]

with open(template_file_path, "r") as template_file:
    template = Template(template_file.read())

    substitutes = dict()
    substitutes["product_code"] = "'{}'".format(product_code)
    substitute = template.safe_substitute(substitutes)

    print("\n================== Generated package file ==================\n")
    print(substitute)
    print("\n============================================================\n")

with open(template_file_path, "w") as template_file:
    template_file.write(substitute)
