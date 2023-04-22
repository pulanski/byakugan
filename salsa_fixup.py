# Search and replace the line `    crate = "salsa_2022",` with `    crate = "salsa",`
# within the `third-party/BUCK` file.


import os

file_path = "third-party/BUCK"

# Make sure the file exists before proceeding
if os.path.isfile(file_path):
    # Read the file contents
    with open(file_path, "r") as file:
        lines = file.readlines()

    # Replace the target line
    with open(file_path, "w") as file:
        for line in lines:
            if line.strip() == 'crate = "salsa_2022",':
                line = '    crate = "salsa",\n'
            file.write(line)

else:
    print(f"The specified file '{file_path}' does not exist.")
