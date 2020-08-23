import hashlib
import sys
from string import Template

args = sys.argv
deployment_file_path = args[1]
version = args[2]
template_file_path = args[3]
generated_file_path = args[4]

# SHA512, SHA256, or SHA1
hash_type = args[5]

print("Generating package for file: %s" % deployment_file_path)
print("     VERSION: %s" % version)
print("     TEMPLATE PATH: %s" % template_file_path)
print("     SAVING AT: %s" % generated_file_path)
print("     USING HASH TYPE: %s" % hash_type)


with open(deployment_file_path, "rb") as deployment_file:
    if str.lower(hash_type) == "sha512":
        deployment_hash = hashlib.sha512(deployment_file.read()).hexdigest()
    elif str.lower(hash_type) == "sha256":
        deployment_hash = hashlib.sha256(deployment_file.read()).hexdigest()
    elif str.lower(hash_type) == "sha1":
        deployment_hash = hashlib.sha1(deployment_file.read()).hexdigest()
    else:
        print('Unsupported hash format "%s".  Please use SHA512, SHA256, or SHA1.', hash_type)
        exit(1)

    print("Generated hash: %s" % str(deployment_hash))

    with open(template_file_path, "r") as template_file:
        template = Template(template_file.read())
        substitute = template.safe_substitute(version=version, hash=deployment_hash)
        print("\n================== Generated package file ==================\n")
        print(substitute)
        print("\n============================================================\n")

        with open(generated_file_path, "w") as generated_file:
            generated_file.write(substitute)

